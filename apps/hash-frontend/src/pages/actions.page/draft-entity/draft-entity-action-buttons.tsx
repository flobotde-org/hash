import { CloseIcon } from "@hashintel/design-system";
import type { Entity } from "@local/hash-graph-sdk/entity";
import type { EntityRootType, Subgraph } from "@local/hash-subgraph";
import { Box, buttonClasses } from "@mui/material";
import type { FunctionComponent } from "react";

import { CheckRegularIcon } from "../../../shared/icons/check-regular-icon";
import { AcceptDraftEntityButton } from "../../shared/accept-draft-entity-button";
import { DiscardDraftEntityButton } from "../../shared/discard-draft-entity-button";
import { useDraftEntities } from "../draft-entities-context";

export const DraftEntityActionButtons: FunctionComponent<{
  entity: Entity;
  subgraph: Subgraph<EntityRootType>;
}> = ({ entity, subgraph }) => {
  const { refetch } = useDraftEntities();

  return (
    <Box marginLeft={1} display="flex" columnGap={1}>
      <DiscardDraftEntityButton
        draftEntity={entity}
        draftEntitySubgraph={subgraph}
        onDiscardedEntity={refetch}
        size="xs"
        variant="tertiary"
        startIcon={<CloseIcon />}
        sx={{
          background: ({ palette }) => palette.gray[20],
          borderColor: ({ palette }) => palette.gray[30],
          color: ({ palette }) => palette.common.black,
          [`> .${buttonClasses.startIcon} > svg`]: {
            fill: ({ palette }) => palette.common.black,
          },
          "&:hover": {
            background: ({ palette }) => palette.gray[30],
          },
        }}
      >
        Ignore
      </DiscardDraftEntityButton>
      <AcceptDraftEntityButton
        draftEntity={entity}
        draftEntitySubgraph={subgraph}
        size="xs"
        variant="primary"
        startIcon={<CheckRegularIcon />}
        onAcceptedEntity={refetch}
      >
        Accept
      </AcceptDraftEntityButton>
    </Box>
  );
};
