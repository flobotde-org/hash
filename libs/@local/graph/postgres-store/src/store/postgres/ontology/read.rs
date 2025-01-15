use alloc::borrow::Cow;

use error_stack::{Report, ResultExt as _};
use futures::{Stream, StreamExt as _};
use hash_graph_store::{
    entity_type::EntityTypeQueryPath,
    error::QueryError,
    filter::Filter,
    subgraph::{
        edges::GraphResolveDepths,
        temporal_axes::{QueryTemporalAxes, VariableAxis},
    },
};
use hash_graph_temporal_versioning::RightBoundedTemporalInterval;
use hash_graph_types::ontology::EntityTypeWithMetadata;
use postgres_types::Json;
use tokio_postgres::GenericClient as _;
use type_system::{
    schema::{ClosedEntityType, EntityTypeUuid, OntologyTypeUuid},
    url::VersionedUrl,
};

use crate::store::postgres::{
    AsClient, PostgresStore,
    query::{
        Distinctness, ForeignKeyReference, ReferenceTable, SelectCompiler, Table, Transpile as _,
        table::DatabaseColumn as _,
    },
};

#[derive(Debug, Default)]
pub struct OntologyTypeTraversalData {
    ontology_ids: Vec<OntologyTypeUuid>,
    resolve_depths: Vec<GraphResolveDepths>,
    traversal_intervals: Vec<RightBoundedTemporalInterval<VariableAxis>>,
}

impl OntologyTypeTraversalData {
    pub fn push(
        &mut self,
        ontology_id: OntologyTypeUuid,
        resolve_depth: GraphResolveDepths,
        traversal_interval: RightBoundedTemporalInterval<VariableAxis>,
    ) {
        self.ontology_ids.push(ontology_id);
        self.resolve_depths.push(resolve_depth);
        self.traversal_intervals.push(traversal_interval);
    }
}

pub struct OntologyEdgeTraversal<L, R> {
    pub left_endpoint: L,
    pub right_endpoint: R,
    pub right_endpoint_ontology_id: OntologyTypeUuid,
    pub resolve_depths: GraphResolveDepths,
    pub traversal_interval: RightBoundedTemporalInterval<VariableAxis>,
}

impl<C: AsClient, A: Send + Sync> PostgresStore<C, A> {
    #[tracing::instrument(level = "trace", skip(self, filter))]
    pub(crate) async fn read_closed_schemas<'f>(
        &self,
        filter: &Filter<'f, EntityTypeWithMetadata>,
        temporal_axes: Option<&'f QueryTemporalAxes>,
    ) -> Result<
        impl Stream<Item = Result<(EntityTypeUuid, ClosedEntityType), Report<QueryError>>>,
        Report<QueryError>,
    > {
        let mut compiler = SelectCompiler::new(temporal_axes, false);

        let ontology_id_index = compiler.add_distinct_selection_with_ordering(
            &EntityTypeQueryPath::OntologyId,
            Distinctness::Distinct,
            None,
        );
        let closed_schema_index =
            compiler.add_selection_path(&EntityTypeQueryPath::ClosedSchema(None));

        compiler.add_filter(filter).change_context(QueryError)?;
        let (statement, parameters) = compiler.compile();

        Ok(self
            .as_client()
            .query_raw(&statement, parameters.iter().copied())
            .await
            .change_context(QueryError)?
            .map(move |row| {
                let row = row.change_context(QueryError)?;
                let Json(schema) = row.get(closed_schema_index);
                Ok((row.get(ontology_id_index), schema))
            }))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) async fn read_ontology_edges<'r, L, R>(
        &self,
        record_ids: &'r OntologyTypeTraversalData,
        reference_table: ReferenceTable,
    ) -> Result<
        impl Iterator<Item = (OntologyTypeUuid, OntologyEdgeTraversal<L, R>)> + 'r,
        Report<QueryError>,
    >
    where
        L: From<VersionedUrl>,
        R: From<VersionedUrl>,
    {
        let table = Table::Reference(reference_table).transpile_to_string();
        let source =
            if let ForeignKeyReference::Single { join, .. } = reference_table.source_relation() {
                join.to_expression(None).transpile_to_string()
            } else {
                unreachable!("Ontology reference tables don't have multiple conditions")
            };
        let target =
            if let ForeignKeyReference::Single { on, .. } = reference_table.target_relation() {
                on.to_expression(None).transpile_to_string()
            } else {
                unreachable!("Ontology reference tables don't have multiple conditions")
            };

        let depth = reference_table
            .inheritance_depth_column()
            .and_then(|column| Some((column.as_str(), column.inheritance_depth()?)));

        let where_statement = match depth {
            Some((column, depth)) => Cow::Owned(format!("WHERE {table}.{column} <= {depth}")),
            _ => Cow::Borrowed(""),
        };

        Ok(self
            .client
            .as_client()
            .query(
                &format!(
                    "
                        SELECT
                            filter.idx         AS idx,
                            source.base_url    AS source_base_url,
                            source.version     AS source_version,
                            target.base_url    AS target_base_url,
                            target.version     AS target_version,
                            target.ontology_id AS target_ontology_id
                        FROM {table}

                        JOIN ontology_ids as source
                          ON {source} = source.ontology_id

                        JOIN unnest($1::uuid[])
                             WITH ORDINALITY AS filter(id, idx)
                          ON filter.id = source.ontology_id

                        JOIN ontology_ids as target
                          ON {target} = target.ontology_id

                        {where_statement};
                    "
                ),
                &[&record_ids.ontology_ids],
            )
            .await
            .change_context(QueryError)?
            .into_iter()
            .map(|row| {
                let index = usize::try_from(row.get::<_, i64>(0) - 1).unwrap_or_else(|error| {
                    // The index is always a valid `usize` because it is the index of the
                    // `record_ids` vectors that was just passed in.
                    unreachable!("invalid index: {error}")
                });
                let right_endpoint_ontology_id = row.get(5);
                (right_endpoint_ontology_id, OntologyEdgeTraversal {
                    left_endpoint: L::from(VersionedUrl {
                        base_url: row.get(1),
                        version: row.get(2),
                    }),
                    right_endpoint: R::from(VersionedUrl {
                        base_url: row.get(3),
                        version: row.get(4),
                    }),
                    right_endpoint_ontology_id,
                    resolve_depths: record_ids.resolve_depths[index],
                    traversal_interval: record_ids.traversal_intervals[index],
                })
            }))
    }
}
