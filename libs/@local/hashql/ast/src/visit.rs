//! A `Visitor` represents an AST modification; it accepts an AST piece and
//! mutates it in place.
//!
//! The visitor works by traversing the entire AST structure recursively. For each
//! node type, there's a corresponding `visit_*` method that can be overridden to
//! perform custom modifications.
//!
//! # Method Formats
//!
//! Methods in the `Visitor` trait follow one of three patterns:
//!
//! * `fn visit_t(&mut self, t: &mut T)` - Most common format for in-place modification
//! * `fn flat_map_t(&mut self, t: T) -> Vec<T>` - For transformations that might produce multiple
//!   nodes
//! * `fn filter_map_t(&mut self, t: T) -> Option<T>` - For transformations that might remove nodes
//!
//! Future extensions to this trait will likely add more methods following these patterns
//! to support advanced AST transformations like inlining function calls or optimizing expressions.
//!
//! # Implementation Notes
//!
//! When writing visitor methods, it is better to use destructuring like this:
//!
//! ```ignore
//! fn visit_struct_field(
//!     &mut self,
//!     StructField {
//!         id,
//!         span,
//!         name,
//!         r#type,
//!     }: &mut StructField,
//! ) {
//!     visit_id(id);
//!     visit_span(span);
//!     visit_ident(name);
//!     visit_type(r#type);
//! }
//! ```
//!
//! than to use field access like this:
//!
//! ```ignore
//! fn visit_struct_field(&mut self, field: &mut StructField) {
//!     visit_id(&mut field.id);
//!     visit_span(&mut field.span);
//!     visit_ident(&mut field.name);
//!     visit_type(&mut field.r#type);
//! }
//! ```
//!
//! The destructuring version is more concise and makes it explicit which fields are being
//! processed. Additionally, if a new field is added to `StructField` in the future, the
//! destructuring pattern will cause a compile error.
//!
//! # Examples
//!
//! ```ignore
//! struct MyVisitor;
//!
//! impl Visitor for MyVisitor {
//!     fn visit_expr(&mut self, expr: &mut Expr) {
//!         // Custom logic before recursion
//!
//!         // Call the default implementation to recursively visit children
//!         walk_expr(self, expr);
//!
//!         // Custom logic after recursion
//!     }
//! }
//! ```
//!
//! The `Visitor` trait is designed to be extended with additional functionality
//! through its default implementations. Each method typically calls a corresponding
//! `walk_*` function that recursively visits the node's children.
use hashql_core::{span::SpanId, symbol::Ident};

use crate::node::{
    expr::{
        CallExpr, ClosureExpr, DictExpr, Expr, ExprKind, FieldExpr, IfExpr, IndexExpr, InputExpr,
        LetExpr, ListExpr, LiteralExpr, NewTypeExpr, StructExpr, TupleExpr, TypeExpr, UseExpr,
        call::{Argument, LabeledArgument},
        closure::{ClosureParam, ClosureSig},
        dict::DictEntry,
        list::ListElement,
        r#struct::StructEntry,
        tuple::TupleElement,
        r#use::{Glob, UseBinding, UseKind},
    },
    generic::{GenericArgument, GenericParam, Generics},
    id::NodeId,
    path::{Path, PathSegment},
    r#type::{
        IntersectionType, StructField, StructType, TupleField, TupleType, Type, TypeKind, UnionType,
    },
};

/// Visitor trait for traversing and modifying the AST.
///
/// To implement a custom AST transformation, create a type that implements this trait
/// and override the methods for the specific node types you want to modify.
///
/// The default implementations call corresponding `walk_*` functions
/// that recursively traverse the AST structure, allowing you to focus only on the
/// specific nodes you want to transform.
///
/// # Method Formats
///
/// This trait currently uses the `visit_*(&mut self, &mut T)` pattern for in-place
/// modifications. Future extensions may include:
///
/// - `flat_map_*(&mut self, T) -> Vec<T>` methods for one-to-many transformations
/// - `filter_map_*(&mut self, T) -> Option<T>` methods for optional transformations
///
/// These additional method types are essential for operations like function inlining
/// (which might produce multiple expressions) or dead code elimination (which might remove nodes).
///
/// # Implementation Strategy
///
/// When implementing a `Visitor`, follow these patterns:
///
/// - To replace a node with a new one, modify its fields directly
/// - To recursively process child nodes, call the corresponding `walk_*` function
/// - To skip processing child nodes, don't call the `walk_*` function
pub trait Visitor {
    #[expect(unused_variables, reason = "trait definition")]
    fn visit_id(&mut self, id: &mut NodeId) {
        // do nothing, no fields to walk
    }

    #[expect(unused_variables, reason = "trait definition")]
    fn visit_span(&mut self, span: &mut SpanId) {
        // do nothing, no fields to walk
    }

    fn visit_ident(&mut self, ident: &mut Ident) {
        walk_ident(self, ident);
    }

    fn visit_path(&mut self, path: &mut Path) {
        walk_path(self, path);
    }

    fn visit_path_segment(&mut self, segment: &mut PathSegment) {
        walk_path_segment(self, segment);
    }

    fn visit_generic_argument(&mut self, argument: &mut GenericArgument) {
        walk_generic_argument(self, argument);
    }

    fn visit_type(&mut self, r#type: &mut Type) {
        walk_type(self, r#type);
    }

    fn visit_tuple_type(&mut self, r#type: &mut TupleType) {
        walk_tuple_type(self, r#type);
    }

    fn visit_tuple_type_field(&mut self, field: &mut TupleField) {
        walk_tuple_type_field(self, field);
    }

    fn visit_struct_type(&mut self, r#type: &mut StructType) {
        walk_struct_type(self, r#type);
    }

    fn visit_struct_type_field(&mut self, field: &mut StructField) {
        walk_struct_type_field(self, field);
    }

    fn visit_union_type(&mut self, r#type: &mut UnionType) {
        walk_union_type(self, r#type);
    }

    fn visit_intersection_type(&mut self, r#type: &mut IntersectionType) {
        walk_intersection_type(self, r#type);
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr(self, expr);
    }

    fn visit_call_expr(&mut self, expr: &mut CallExpr) {
        walk_call_expr(self, expr);
    }

    fn visit_argument(&mut self, argument: &mut Argument) {
        walk_argument(self, argument);
    }

    fn visit_labeled_argument(&mut self, labeled_argument: &mut LabeledArgument) {
        walk_labeled_argument(self, labeled_argument);
    }

    fn visit_struct_expr(&mut self, expr: &mut StructExpr) {
        walk_struct_expr(self, expr);
    }

    fn visit_struct_expr_entry(&mut self, entry: &mut StructEntry) {
        walk_struct_expr_entry(self, entry);
    }

    fn visit_dict_expr(&mut self, expr: &mut DictExpr) {
        walk_dict_expr(self, expr);
    }

    fn visit_dict_expr_entry(&mut self, entry: &mut DictEntry) {
        walk_dict_expr_entry(self, entry);
    }

    fn visit_tuple_expr(&mut self, expr: &mut TupleExpr) {
        walk_tuple_expr(self, expr);
    }

    fn visit_tuple_expr_element(&mut self, element: &mut TupleElement) {
        walk_tuple_expr_element(self, element);
    }

    fn visit_list_expr(&mut self, expr: &mut ListExpr) {
        walk_list_expr(self, expr);
    }

    fn visit_list_expr_element(&mut self, element: &mut ListElement) {
        walk_list_expr_element(self, element);
    }

    fn visit_literal_expr(&mut self, expr: &mut LiteralExpr) {
        walk_literal_expr(self, expr);
    }

    fn visit_let_expr(&mut self, expr: &mut LetExpr) {
        walk_let_expr(self, expr);
    }

    fn visit_type_expr(&mut self, expr: &mut TypeExpr) {
        walk_type_expr(self, expr);
    }

    fn visit_newtype_expr(&mut self, expr: &mut NewTypeExpr) {
        walk_newtype_expr(self, expr);
    }

    fn visit_use_expr(&mut self, expr: &mut UseExpr) {
        walk_use_expr(self, expr);
    }

    fn visit_use_expr_binding(&mut self, binding: &mut UseBinding) {
        walk_use_expr_binding(self, binding);
    }

    fn visit_use_expr_glob(&mut self, glob: &mut Glob) {
        walk_use_expr_glob(self, glob);
    }

    fn visit_input_expr(&mut self, expr: &mut InputExpr) {
        walk_input_expr(self, expr);
    }

    fn visit_closure_expr(&mut self, expr: &mut ClosureExpr) {
        walk_closure_expr(self, expr);
    }

    fn visit_closure_sig(&mut self, sig: &mut ClosureSig) {
        walk_closure_sig(self, sig);
    }

    fn visit_closure_param(&mut self, param: &mut ClosureParam) {
        walk_closure_param(self, param);
    }

    fn visit_generics(&mut self, generics: &mut Generics) {
        walk_generics(self, generics);
    }

    fn visit_generic_param(&mut self, param: &mut GenericParam) {
        walk_generic_param(self, param);
    }

    fn visit_if_expr(&mut self, expr: &mut IfExpr) {
        walk_if_expr(self, expr);
    }

    fn visit_field_expr(&mut self, expr: &mut FieldExpr) {
        walk_field_expr(self, expr);
    }
    fn visit_index_expr(&mut self, expr: &mut IndexExpr) {
        walk_index_expr(self, expr);
    }
}

pub fn walk_ident<T: Visitor + ?Sized>(
    visitor: &mut T,
    Ident {
        name: _,
        span,
        kind: _,
    }: &mut Ident,
) {
    visitor.visit_span(span);
}

pub fn walk_type<T: Visitor + ?Sized>(visitor: &mut T, Type { id, span, kind }: &mut Type) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    match kind {
        TypeKind::Unknown | TypeKind::Infer | TypeKind::Never => {}
        TypeKind::Path(path) => visitor.visit_path(path),
        TypeKind::Tuple(tuple_type) => visitor.visit_tuple_type(tuple_type),
        TypeKind::Struct(struct_type) => visitor.visit_struct_type(struct_type),
        TypeKind::Union(union_type) => visitor.visit_union_type(union_type),
        TypeKind::Intersection(intersection_type) => {
            visitor.visit_intersection_type(intersection_type);
        }
    }
}

pub fn walk_tuple_type<T: Visitor + ?Sized>(
    visitor: &mut T,
    TupleType { id, span, fields }: &mut TupleType,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    for field in fields {
        visitor.visit_tuple_type_field(field);
    }
}

pub fn walk_tuple_type_field<T: Visitor + ?Sized>(
    visitor: &mut T,
    TupleField { id, span, r#type }: &mut TupleField,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_type(r#type);
}

pub fn walk_struct_type<T: Visitor + ?Sized>(
    visitor: &mut T,
    StructType { id, span, fields }: &mut StructType,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    for field in fields {
        visitor.visit_struct_type_field(field);
    }
}

pub fn walk_struct_type_field<T: Visitor + ?Sized>(
    visitor: &mut T,
    StructField {
        id,
        span,
        name,
        r#type,
    }: &mut StructField,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);
    visitor.visit_ident(name);
    visitor.visit_type(r#type);
}

pub fn walk_union_type<T: Visitor + ?Sized>(
    visitor: &mut T,
    UnionType { id, span, types }: &mut UnionType,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    for r#type in types {
        visitor.visit_type(r#type);
    }
}

pub fn walk_intersection_type<T: Visitor + ?Sized>(
    visitor: &mut T,
    IntersectionType { id, span, types }: &mut IntersectionType,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    for r#type in types {
        visitor.visit_type(r#type);
    }
}

pub fn walk_expr<T: Visitor + ?Sized>(visitor: &mut T, Expr { id, span, kind }: &mut Expr) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    match kind {
        ExprKind::Call(call_expr) => visitor.visit_call_expr(call_expr),
        ExprKind::Struct(struct_expr) => visitor.visit_struct_expr(struct_expr),
        ExprKind::Dict(dict_expr) => visitor.visit_dict_expr(dict_expr),
        ExprKind::Tuple(tuple_expr) => visitor.visit_tuple_expr(tuple_expr),
        ExprKind::List(list_expr) => visitor.visit_list_expr(list_expr),
        ExprKind::Literal(literal_expr) => visitor.visit_literal_expr(literal_expr),
        ExprKind::Path(path) => visitor.visit_path(path),
        ExprKind::Let(let_expr) => visitor.visit_let_expr(let_expr),
        ExprKind::Type(type_expr) => visitor.visit_type_expr(type_expr),
        ExprKind::NewType(new_type_expr) => visitor.visit_newtype_expr(new_type_expr),
        ExprKind::Use(use_expr) => visitor.visit_use_expr(use_expr),
        ExprKind::Input(input_expr) => visitor.visit_input_expr(input_expr),
        ExprKind::Closure(closure_expr) => visitor.visit_closure_expr(closure_expr),
        ExprKind::If(if_expr) => visitor.visit_if_expr(if_expr),
        ExprKind::Field(field_expr) => visitor.visit_field_expr(field_expr),
        ExprKind::Index(index_expr) => visitor.visit_index_expr(index_expr),
    }
}

pub fn walk_call_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    CallExpr {
        id,
        span,
        function,
        arguments,
        labeled_arguments,
    }: &mut CallExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_expr(function);

    for argument in arguments {
        visitor.visit_argument(argument);
    }

    for labeled_argument in labeled_arguments {
        visitor.visit_labeled_argument(labeled_argument);
    }
}

pub fn walk_argument<T: Visitor + ?Sized>(
    visitor: &mut T,
    Argument { id, span, value }: &mut Argument,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);
    visitor.visit_expr(value);
}

pub fn walk_labeled_argument<T: Visitor + ?Sized>(
    visitor: &mut T,
    LabeledArgument {
        id,
        span,
        label,
        value,
    }: &mut LabeledArgument,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);
    visitor.visit_ident(label);
    visitor.visit_argument(value);
}

pub fn walk_struct_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    StructExpr {
        id,
        span,
        entries,
        r#type,
    }: &mut StructExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    if let Some(r#type) = r#type {
        visitor.visit_type(r#type);
    }

    for entry in entries {
        visitor.visit_struct_expr_entry(entry);
    }
}

pub fn walk_struct_expr_entry<T: Visitor + ?Sized>(
    visitor: &mut T,
    StructEntry {
        id,
        span,
        key,
        value,
    }: &mut StructEntry,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);
    visitor.visit_ident(key);
    visitor.visit_expr(value);
}

pub fn walk_dict_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    DictExpr {
        id,
        span,
        entries,
        r#type,
    }: &mut DictExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    if let Some(r#type) = r#type {
        visitor.visit_type(r#type);
    }

    for entry in entries {
        visitor.visit_dict_expr_entry(entry);
    }
}

pub fn walk_dict_expr_entry<T: Visitor + ?Sized>(
    visitor: &mut T,
    DictEntry {
        id,
        span,
        key,
        value,
    }: &mut DictEntry,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);
    visitor.visit_expr(key);
    visitor.visit_expr(value);
}

pub fn walk_tuple_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    TupleExpr {
        id,
        span,
        elements,
        r#type,
    }: &mut TupleExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    if let Some(r#type) = r#type {
        visitor.visit_type(r#type);
    }

    for element in elements {
        visitor.visit_tuple_expr_element(element);
    }
}

pub fn walk_tuple_expr_element<T: Visitor + ?Sized>(
    visitor: &mut T,
    TupleElement { id, span, value }: &mut TupleElement,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);
    visitor.visit_expr(value);
}

pub fn walk_list_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    ListExpr {
        id,
        span,
        elements,
        r#type,
    }: &mut ListExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    if let Some(r#type) = r#type {
        visitor.visit_type(r#type);
    }

    for element in elements {
        visitor.visit_list_expr_element(element);
    }
}

pub fn walk_list_expr_element<T: Visitor + ?Sized>(
    visitor: &mut T,
    ListElement { id, span, value }: &mut ListElement,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);
    visitor.visit_expr(value);
}

pub fn walk_literal_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    LiteralExpr {
        id,
        span,
        kind: _,
        r#type,
    }: &mut LiteralExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    if let Some(r#type) = r#type {
        visitor.visit_type(r#type);
    }
}

pub fn walk_let_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    LetExpr {
        id,
        span,
        name,
        value,
        r#type,
        body,
    }: &mut LetExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_ident(name);
    visitor.visit_expr(value);

    if let Some(r#type) = r#type {
        visitor.visit_type(r#type);
    }

    visitor.visit_expr(body);
}

pub fn walk_type_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    TypeExpr {
        id,
        span,
        name,
        value,
        body,
    }: &mut TypeExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_ident(name);
    visitor.visit_type(value);
    visitor.visit_expr(body);
}

pub fn walk_newtype_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    NewTypeExpr {
        id,
        span,
        name,
        value,
        body,
    }: &mut NewTypeExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_ident(name);
    visitor.visit_type(value);
    visitor.visit_expr(body);
}

pub fn walk_use_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    UseExpr {
        id,
        span,
        path,
        kind,
        body,
    }: &mut UseExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_path(path);

    match kind {
        UseKind::Named(bindings) => {
            for binding in bindings {
                visitor.visit_use_expr_binding(binding);
            }
        }
        UseKind::Glob(glob) => visitor.visit_use_expr_glob(glob),
    }

    visitor.visit_expr(body);
}

pub fn walk_use_expr_binding<T: Visitor + ?Sized>(
    visitor: &mut T,
    UseBinding {
        id,
        span,
        name,
        alias,
    }: &mut UseBinding,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_ident(name);

    if let Some(alias) = alias {
        visitor.visit_ident(alias);
    }
}

pub fn walk_use_expr_glob<T: Visitor + ?Sized>(visitor: &mut T, Glob { id, span }: &mut Glob) {
    visitor.visit_id(id);
    visitor.visit_span(span);
}

pub fn walk_input_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    InputExpr {
        id,
        span,
        name,
        r#type,
        default,
    }: &mut InputExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_ident(name);

    if let Some(r#type) = r#type {
        visitor.visit_type(r#type);
    }

    if let Some(default) = default {
        visitor.visit_expr(default);
    }
}

pub fn walk_closure_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    ClosureExpr {
        id,
        span,
        sig,
        body,
    }: &mut ClosureExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_closure_sig(sig);

    visitor.visit_expr(body);
}

pub fn walk_closure_sig<T: Visitor + ?Sized>(
    visitor: &mut T,
    ClosureSig {
        id,
        span,
        generics,
        inputs,
        output,
    }: &mut ClosureSig,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_generics(generics);

    for param in inputs {
        visitor.visit_closure_param(param);
    }

    visitor.visit_type(output);
}

pub fn walk_closure_param<T: Visitor + ?Sized>(
    visitor: &mut T,
    ClosureParam {
        id,
        span,
        name,
        r#type,
    }: &mut ClosureParam,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_ident(name);
    visitor.visit_type(r#type);
}

pub fn walk_generics<T: Visitor + ?Sized>(
    visitor: &mut T,
    Generics { id, span, params }: &mut Generics,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    for param in params {
        visitor.visit_generic_param(param);
    }
}

pub fn walk_generic_param<T: Visitor + ?Sized>(
    visitor: &mut T,
    GenericParam {
        id,
        span,
        name,
        bound,
    }: &mut GenericParam,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_ident(name);

    if let Some(bound) = bound {
        visitor.visit_type(bound);
    }
}

pub fn walk_if_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    IfExpr {
        id,
        span,
        test,
        then,
        r#else,
    }: &mut IfExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_expr(test);
    visitor.visit_expr(then);

    if let Some(r#else) = r#else {
        visitor.visit_expr(r#else);
    }
}

pub fn walk_field_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    FieldExpr {
        id,
        span,
        value,
        field,
    }: &mut FieldExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_expr(value);
    visitor.visit_ident(field);
}

pub fn walk_index_expr<T: Visitor + ?Sized>(
    visitor: &mut T,
    IndexExpr {
        id,
        span,
        value,
        index,
    }: &mut IndexExpr,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_expr(value);
    visitor.visit_expr(index);
}

pub fn walk_path<T: Visitor + ?Sized>(
    visitor: &mut T,
    Path {
        id,
        span,
        rooted: _,
        segments,
    }: &mut Path,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    for segment in segments {
        visitor.visit_path_segment(segment);
    }
}

pub fn walk_path_segment<T: Visitor + ?Sized>(
    visitor: &mut T,
    PathSegment {
        id,
        span,
        name,
        arguments,
    }: &mut PathSegment,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_ident(name);

    for argument in arguments {
        visitor.visit_generic_argument(argument);
    }
}

pub fn walk_generic_argument<T: Visitor + ?Sized>(
    visitor: &mut T,
    GenericArgument { id, span, r#type }: &mut GenericArgument,
) {
    visitor.visit_id(id);
    visitor.visit_span(span);

    visitor.visit_type(r#type);
}
