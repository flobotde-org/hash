import { DbBlock, DbPage } from "../types/dbTypes";

import { Visibility } from "../graphql/autoGeneratedTypes";
import { randomTimes } from "./util";

export const pages = (() => {
  const blockData: DbBlock[] = [
    {
      id: "b1",
      properties: {
        componentId: "https://block.blockprotocol.org/header",
        entityType: "Text",
        entityId: "text1",
      },
      ...randomTimes(),
      createdById: "2",
      namespaceId: "2",
      type: "Block",
      visibility: Visibility.Public,
    },
    {
      properties: {
        componentId: "https://block.blockprotocol.org/paragraph",
        entityType: "Text",
        entityId: "text2",
      },
      id: "b2",
      ...randomTimes(),
      createdById: "2",
      namespaceId: "2",
      type: "Block",
      visibility: Visibility.Public,
    },
    {
      properties: {
        componentId: "https://block.blockprotocol.org/paragraph",
        entityType: "Text",
        entityId: "text3",
      },
      id: "b3",
      ...randomTimes(),
      createdById: "2",
      namespaceId: "2",
      type: "Block",
      visibility: Visibility.Public,
    },
    {
      properties: {
        componentId: "https://block.blockprotocol.org/table",
        entityType: "Table",
        entityId: "t1",
      },
      id: "b4",
      ...randomTimes(),
      createdById: "2",
      namespaceId: "2",
      type: "Block",
      visibility: Visibility.Public,
    },
    {
      properties: {
        componentId: "https://block.blockprotocol.org/header",
        entityType: "Text",
        entityId: "text5",
      },
      id: "b5",
      ...randomTimes(),
      createdById: "2",
      namespaceId: "6",
      type: "Block",
      visibility: Visibility.Public,
    },
    {
      properties: {
        componentId: "https://block.blockprotocol.org/paragraph",
        entityType: "Text",
        entityId: "text2",
      },
      id: "b6",
      ...randomTimes(),
      createdById: "2",
      namespaceId: "6",
      type: "Block",
      visibility: Visibility.Public,
    },
    {
      properties: {
        componentId: "https://block.blockprotocol.org/paragraph",
        entityType: "Text",
        entityId: "text3",
      },
      id: "b7",
      ...randomTimes(),
      createdById: "2",
      namespaceId: "6",
      type: "Block",
      visibility: Visibility.Public,
    },
    {
      properties: {
        componentId: "https://block.blockprotocol.org/paragraph",
        entityType: "Text",
        entityId: "text4",
      },
      id: "b8",
      ...randomTimes(),
      createdById: "2",
      namespaceId: "6",
      type: "Block",
      visibility: Visibility.Public,
    },
  ];

  const pageData: DbPage[] = [
    {
      id: "page1",
      type: "Page",
      namespaceId: "2",
      createdById: "2",
      ...randomTimes(),
      properties: {
        contents: [
          {
            entityId: "b1"
          },
          {
            entityId: "b2"
          },
          {
            entityId: "b3"
          },
          {
            entityId: "b4"
          }
        ],
        title: "My awesome page",
      },
      visibility: Visibility.Public,
    },
    {
      id: "page2",
      type: "Page",
      namespaceId: "6",
      createdById: "2",
      ...randomTimes(),
      properties: {
        contents: [
          {
            entityId: "b5"
          },
          {
            entityId: "b6"
          },
          {
            entityId: "b7"
          },
          {
            entityId: "b8"
          }
        ],
        title: "HASH's 1st page",
      },
      visibility: Visibility.Public,
    },
  ];
  return [...blockData, ...pageData];
})();
