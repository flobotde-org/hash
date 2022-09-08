import { ApolloError } from "apollo-server-express";
import { AxiosError } from "axios";

import {
  PersistedPropertyType,
  MutationCreatePropertyTypeArgs,
  MutationUpdatePropertyTypeArgs,
  QueryGetPropertyTypeArgs,
  ResolverFn,
} from "../../apiTypes.gen";
import { LoggedInGraphQLContext } from "../../context";
import { PropertyTypeModel } from "../../../model";
import { propertyTypeModelToGQL } from "./model-mapping";

export const createPropertyType: ResolverFn<
  Promise<PersistedPropertyType>,
  {},
  LoggedInGraphQLContext,
  MutationCreatePropertyTypeArgs
> = async (_, params, { dataSources, user }) => {
  const { graphApi } = dataSources;
  const { accountId, propertyType } = params;

  const createdPropertyTypeModel = await PropertyTypeModel.create(graphApi, {
    accountId: accountId ?? user.getAccountId(),
    schema: propertyType,
  }).catch((err) => {
    throw new ApolloError(err, "CREATION_ERROR");
  });

  return propertyTypeModelToGQL(createdPropertyTypeModel);
};

export const getAllLatestPropertyTypes: ResolverFn<
  Promise<PersistedPropertyType[]>,
  {},
  LoggedInGraphQLContext,
  {}
> = async (_, __, { dataSources, user }) => {
  const { graphApi } = dataSources;

  const allLatestPropertyTypeModels = await PropertyTypeModel.getAllLatest(
    graphApi,
    {
      accountId: user.getAccountId(),
    },
  ).catch((err: AxiosError) => {
    throw new ApolloError(
      `Unable to retrieve all latest property types. ${err.response?.data}`,
      "GET_ALL_ERROR",
    );
  });

  return allLatestPropertyTypeModels.map(propertyTypeModelToGQL);
};

export const getPropertyType: ResolverFn<
  Promise<PersistedPropertyType>,
  {},
  LoggedInGraphQLContext,
  QueryGetPropertyTypeArgs
> = async (_, { propertyTypeVersionedUri }, { dataSources }) => {
  const { graphApi } = dataSources;

  const propertyTypeModel = await PropertyTypeModel.get(graphApi, {
    versionedUri: propertyTypeVersionedUri,
  }).catch((err: AxiosError) => {
    throw new ApolloError(
      `Unable to retrieve property type. ${err.response?.data}`,
      "GET_ERROR",
    );
  });

  return propertyTypeModelToGQL(propertyTypeModel);
};

export const updatePropertyType: ResolverFn<
  Promise<PersistedPropertyType>,
  {},
  LoggedInGraphQLContext,
  MutationUpdatePropertyTypeArgs
> = async (_, params, { dataSources, user }) => {
  const { graphApi } = dataSources;
  const { accountId, propertyTypeVersionedUri, updatedPropertyType } = params;

  const propertyTypeModel = await PropertyTypeModel.get(graphApi, {
    versionedUri: propertyTypeVersionedUri,
  }).catch((err: AxiosError) => {
    throw new ApolloError(
      `Unable to retrieve property type. ${err.response?.data} [URI=${propertyTypeVersionedUri}]`,
      "GET_ERROR",
    );
  });

  const updatedPropertyTypeModel = await propertyTypeModel
    .update(graphApi, {
      accountId: accountId ?? user.getAccountId(),
      schema: updatedPropertyType,
    })
    .catch((err: AxiosError) => {
      const msg =
        err.response?.status === 409
          ? `Property type URI doesn't exist, unable to update. [URI=${propertyTypeVersionedUri}]`
          : `Couldn't update property type.`;

      throw new ApolloError(msg, "CREATION_ERROR");
    });

  return propertyTypeModelToGQL(updatedPropertyTypeModel);
};
