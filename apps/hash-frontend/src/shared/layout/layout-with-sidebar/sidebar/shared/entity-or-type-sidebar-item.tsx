import type { EntityType } from "@blockprotocol/type-system";
import { extractBaseUrl } from "@blockprotocol/type-system";
import { EntityOrTypeIcon, IconButton } from "@hashintel/design-system";
import { pluralize } from "@local/hash-isomorphic-utils/pluralize";
import type { BoxProps } from "@mui/material";
import { Box, styled, Tooltip, Typography } from "@mui/material";
import { bindTrigger, usePopupState } from "material-ui-popup-state/hooks";
import { useRouter } from "next/router";
import type { FunctionComponent } from "react";
import { useRef } from "react";

import { useEntityTypesContextRequired } from "../../../../entity-types-context/hooks/use-entity-types-context-required";
import { EllipsisRegularIcon } from "../../../../icons/ellipsis-regular-icon";
import { Link } from "../../../../ui";
import { EntityMenu } from "./entity-or-type-sidebar-item/entity-menu";
import { EntityTypeMenu } from "./entity-or-type-sidebar-item/entity-type-menu";

const Container = styled((props: BoxProps & { selected: boolean }) => (
  <Box component="li" {...props} />
))(({ theme, selected }) => ({
  paddingLeft: theme.spacing(2),
  paddingRight: theme.spacing(1),
  paddingTop: theme.spacing(0.5),
  paddingBottom: theme.spacing(0.5),
  margin: `0 ${theme.spacing(0.5)}`,
  borderRadius: "4px",
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
  color: theme.palette.gray[70],

  "&:hover, &:focus": {
    backgroundColor: selected ? theme.palette.gray[20] : theme.palette.gray[10],
    color: selected ? theme.palette.gray[80] : theme.palette.gray[70],

    "& .entity-menu-trigger": {
      color: theme.palette.gray[80],
    },
  },

  ...(selected && {
    backgroundColor: theme.palette.gray[30],
    color: theme.palette.gray[80],
  }),

  "&:focus-within": {
    backgroundColor: theme.palette.gray[20],
    color: theme.palette.gray[80],
  },
}));

export const EntityOrTypeSidebarItem: FunctionComponent<{
  entityType: EntityType;
  href?: string;
  variant: "entity" | "entity-type";
}> = ({ entityType, href, variant }) => {
  const {
    title,
    titlePlural: maybeTitlePlural,
    $id: entityTypeId,
    icon,
  } = entityType;

  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing -- we don't want an empty string
  const titlePlural = maybeTitlePlural || pluralize(title);

  const entityMenuTriggerRef = useRef(null);

  const popupState = usePopupState({
    variant: "popover",
    popupId: "entity-menu",
  });

  const router = useRouter();

  const baseUrl = extractBaseUrl(entityTypeId);
  const url = new URL(`${window.location.origin}${router.asPath}/`);
  const urlBase = `${url.origin}${url.pathname.replace(/\/$/, "")}/`;
  const selected =
    router.route === "/[shortname]/types/entity-type/[entity-type-id]" &&
    urlBase === baseUrl;

  const { isSpecialEntityTypeLookup } = useEntityTypesContextRequired();

  const { isLink } = isSpecialEntityTypeLookup?.[entityTypeId] ?? {};

  return (
    <Container component="li" tabIndex={0} selected={selected} minHeight={32}>
      <Link
        tabIndex={-1}
        sx={{ flex: 1 }}
        noLinkStyle
        href={href ?? baseUrl}
        flex={1}
      >
        <Typography
          variant="smallTextLabels"
          sx={{
            alignItems: "center",
            display: "flex",
            color: ({ palette }) => palette.gray[70],
            fontWeight: 500,
          }}
        >
          <Box
            component="span"
            sx={{
              marginRight: 1,
              maxWidth: 18,
              display: "inline-flex",
              justifyContent: "center",
            }}
          >
            <EntityOrTypeIcon
              entity={null}
              icon={icon}
              isLink={!!isLink}
              fontSize={16}
              fill={({ palette }) =>
                variant === "entity-type" ? palette.blue[70] : palette.gray[50]
              }
            />
          </Box>
          {title}
        </Typography>
      </Link>

      <Tooltip title="Options" sx={{ left: 5 }}>
        <IconButton
          ref={entityMenuTriggerRef}
          className="entity-menu-trigger"
          {...bindTrigger(popupState)}
          size="medium"
          unpadded
          sx={({ palette }) => ({
            color: [selected ? palette.gray[80] : "transparent"],
            "&:focus-visible, &:hover": {
              backgroundColor: palette.gray[selected ? 40 : 30],
              color: palette.gray[selected ? 80 : 40],
            },
          })}
        >
          <EllipsisRegularIcon />
        </IconButton>
      </Tooltip>
      {variant === "entity" ? (
        <EntityMenu
          entityTypeId={entityTypeId}
          popupState={popupState}
          title={title}
          titlePlural={titlePlural}
          entityTypeIcon={icon}
          isLinkType={isLink}
        />
      ) : (
        <EntityTypeMenu
          entityTypeId={entityTypeId}
          popupState={popupState}
          title={title}
          titlePlural={titlePlural}
          url={baseUrl}
        />
      )}
    </Container>
  );
};
