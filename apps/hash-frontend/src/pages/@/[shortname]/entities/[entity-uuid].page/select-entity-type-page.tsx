import { faPlus, faWarning } from "@fortawesome/free-solid-svg-icons";
import { Chip, FontAwesomeIcon, WhiteCard } from "@hashintel/design-system";
import { Box, Divider, Typography } from "@mui/material";
import { useRouter } from "next/router";
import { useContext, useState } from "react";

import { useSnackbar } from "../../../../../components/hooks/use-snackbar";
import { Button } from "../../../../../shared/ui";
import { EntityEditorContainer } from "../../../../shared/entity/entity-editor-container";
import { EntityHeader } from "../../../../shared/entity/entity-header";
import { LinksSectionEmptyState } from "../../../../shared/entity/shared/links-section-empty-state";
import { PropertiesSectionEmptyState } from "../../../../shared/entity/shared/properties-section-empty-state";
import { EntityTypeSelector } from "../../../../shared/entity-type-selector";
import { SectionWrapper } from "../../../../shared/section-wrapper";
import { WorkspaceContext } from "../../../../shared/workspace-context";

const selectorOrButtonHeight = 46;

export const SelectEntityTypePage = () => {
  const router = useRouter();
  const { triggerSnackbar } = useSnackbar();
  const [isSelectingType, setIsSelectingType] = useState(false);
  const [loading, setLoading] = useState(false);

  const { activeWorkspace } = useContext(WorkspaceContext);

  if (!activeWorkspace) {
    throw new Error("Active workspace must be set");
  }

  return (
    <>
      <EntityHeader
        entityLabel="New entity"
        isInSlide={false}
        isLocalDraft
        lightTitle
        onDraftArchived={() => {}}
        onDraftPublished={() => {}}
        onUnarchived={() => {}}
      />

      <EntityEditorContainer isInSlide={false}>
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            gap: 6.5,
          }}
        >
          <SectionWrapper
            title="Types"
            titleStartContent={<Chip label="No type" size="xs" />}
          >
            <WhiteCard>
              <Box
                pt={3.75}
                pb={2}
                gap={0.75}
                display="flex"
                flexDirection="column"
                alignItems="center"
                textAlign="center"
              >
                <Typography
                  display="flex"
                  alignItems="center"
                  variant="largeTextLabels"
                  fontWeight={600}
                  gap={1}
                >
                  <FontAwesomeIcon
                    icon={faWarning}
                    sx={{ color: "yellow.80" }}
                  />
                  This entity requires a type
                </Typography>
                <Typography color="gray.60">
                  Types describe an entity, and determine the properties and
                  links that can be associated with it.
                </Typography>
              </Box>

              <Divider />
              <Box
                sx={{
                  display: "flex",
                  gap: 1,
                  alignItems: "center",
                  p: 4,
                  justifyContent: "center",
                  flexWrap: "wrap",
                }}
              >
                {isSelectingType ? (
                  <EntityTypeSelector
                    excludeLinkTypes
                    inputHeight={selectorOrButtonHeight}
                    onCancel={() => setIsSelectingType(false)}
                    onSelect={async (entityType) => {
                      try {
                        setIsSelectingType(false);
                        setLoading(true);

                        await router.push(
                          `/new/entity?entity-type-id=${encodeURIComponent(
                            entityType.schema.$id,
                          )}`,
                        );
                      } catch (error) {
                        triggerSnackbar.error((error as Error).message);
                      } finally {
                        setLoading(false);
                      }
                    }}
                    onCreateNew={(searchValue) => {
                      let href = `/new/types/entity-type`;
                      if (searchValue) {
                        href += `?name=${encodeURIComponent(searchValue)}`;
                      }

                      void router.push(href);
                    }}
                  />
                ) : (
                  <>
                    <Button
                      loading={loading}
                      onClick={() => setIsSelectingType(true)}
                      size="small"
                      startIcon={!loading && <FontAwesomeIcon icon={faPlus} />}
                      sx={{
                        fontSize: 14,
                        paddingX: 2,
                        height: selectorOrButtonHeight,
                      }}
                    >
                      Add a type
                    </Button>
                    {!loading && (
                      <Typography variant="smallTextLabels" fontWeight={600}>
                        to start using this entity
                      </Typography>
                    )}
                  </>
                )}
              </Box>
            </WhiteCard>
          </SectionWrapper>

          <PropertiesSectionEmptyState />

          <LinksSectionEmptyState direction="Outgoing" />

          {/* <PeersSectionEmptyState /> */}
        </Box>
      </EntityEditorContainer>
    </>
  );
};
