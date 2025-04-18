import type { BaseUrl } from "@blockprotocol/type-system";
import { faClose } from "@fortawesome/free-solid-svg-icons";
import type { ButtonProps } from "@hashintel/design-system";
import {
  Button,
  FontAwesomeIcon,
  IconButton,
  Modal,
  TextField,
} from "@hashintel/design-system";
import { fluidFontClassName } from "@hashintel/design-system/theme";
import {
  Box,
  Divider,
  inputLabelClasses,
  Stack,
  Typography,
} from "@mui/material";
import type { PopupState } from "material-ui-popup-state/hooks";
import { bindDialog, bindToggle } from "material-ui-popup-state/hooks";
import type {
  ComponentPropsWithoutRef,
  ComponentPropsWithRef,
  ElementType,
  FormEvent,
  ReactElement,
  ReactNode,
  Ref,
} from "react";
import {
  createElement,
  forwardRef,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { DeepPartial, DefaultValues } from "react-hook-form";
import { FormProvider, useForm, useFormContext } from "react-hook-form";

import type { TitleValidationFunction } from "../../shared/ontology-functions-context";
import { QuestionIcon } from "./question-icon";
import { withHandler } from "./with-handler";

type TypeFormSubmitProps = Omit<
  ButtonProps,
  "size" | "variant" | "disabled" | "type" | "loading"
>;

export const TypeFormTitleField = ({
  fieldDisabled,
  validateTitle,
}: {
  fieldDisabled: boolean;
  validateTitle: (title: string) => ReturnType<TitleValidationFunction>;
}) => {
  const {
    register,
    formState: {
      isSubmitting,
      errors: { name: nameError },
    },
    getValues,
    clearErrors,
  } = useFormContext<TypeFormDefaults>();

  /**
   * Frustratingly, we have to track this ourselves
   * @see https://github.com/react-hook-form/react-hook-form/discussions/2633
   */
  const [titleValid, setTitleValid] = useState(false);

  return (
    <TextField
      fullWidth
      label="Singular name"
      required
      placeholder="e.g. Stock Price"
      disabled={fieldDisabled || isSubmitting}
      {...(!fieldDisabled && {
        error: !!nameError,
        helperText: nameError?.message,
        success: titleValid,
      })}
      {...register("name", {
        required: true,
        onChange() {
          clearErrors("name");
          setTitleValid(false);
        },
        async validate(value) {
          if (fieldDisabled) {
            setTitleValid(true);
            return true;
          }

          const { allowed, message } = await validateTitle(value);

          setTitleValid(getValues("name") === value && allowed);

          return !allowed ? message : true;
        },
      })}
    />
  );
};

// @todo handle this field having a different type than property
export const TypeFormDescriptionField = ({
  defaultValues,
  fieldDisabled,
  kind,
}: {
  defaultValues: { description?: string };
  fieldDisabled: boolean;
  kind: "link" | "property";
}) => {
  const {
    register,
    formState: {
      isSubmitting,
      errors: { description: descriptionError },
      touchedFields: { description: descriptionTouched },
    },
    clearErrors,
  } = useFormContext<TypeFormDefaults>();

  const [descriptionValid, setDescriptionValid] = useState(false);

  /**
   * Some default property types don't have descriptions. We don't want to have
   * to enter one if you pass a preset value for description which is falsey
   *
   * @todo remove this when all property types have descriptions
   */
  const descriptionRequired =
    !("description" in defaultValues) || !!defaultValues.description;

  return (
    <TextField
      multiline
      fullWidth
      inputProps={{ minRows: 1 }}
      label={
        <>
          Description{" "}
          <Box
            sx={{
              order: 1,
              ml: 0.75,
              display: "flex",
              alignItems: "center",
            }}
          >
            <QuestionIcon tooltip="Descriptions help people understand what property types can be used for, and help make them more discoverable (allowing for reuse)." />
          </Box>
        </>
      }
      required={descriptionRequired}
      placeholder={`Describe this ${kind} type in one or two sentences`}
      disabled={fieldDisabled || isSubmitting}
      {...(!fieldDisabled &&
        descriptionTouched && {
          success: descriptionValid,
          error: !!descriptionError,
        })}
      {...register("description", {
        required: descriptionRequired,
        onChange() {
          clearErrors("description");
          setDescriptionValid(false);
        },
        validate(value) {
          const valid = !descriptionRequired || !!value;

          setDescriptionValid(valid);
          return valid ? true : "You must choose a description";
        },
      })}
    />
  );
};

type TypeFormModalProps<T extends ElementType = "div"> =
  ComponentPropsWithoutRef<T> & {
    as?: T;
    popupState: PopupState;
    ref?: Ref<ComponentPropsWithRef<T>["ref"]> | null;
    baseUrl?: BaseUrl;
  };

type PolymorphicProps<P, T extends ElementType> = P & TypeFormModalProps<T>;

type PolymorphicComponent<
  P = Record<string, unknown>,
  D extends ElementType = "div",
> = <T extends ElementType = D>(
  props: PolymorphicProps<P, T>,
) => ReactElement | null;

// @ts-expect-error -- fix this at some point, search 'PolymorphicComponent with forwardRef'
export const TypeFormModal: PolymorphicComponent = forwardRef(
  // @ts-expect-error -- fix this at some point, search 'PolymorphicComponent with forwardRef'
  <T extends ElementType>(
    props: TypeFormModalProps<T>,
    ref: Ref<HTMLElement>,
  ) => {
    const { as = "div", popupState, ...restProps } = props;

    const inner = createElement(as, {
      ...restProps,
      ref,
      popupState,
    });

    return (
      <Modal
        {...bindDialog(popupState)}
        disableEscapeKeyDown
        contentStyle={(theme) => ({
          p: { xs: 0, md: 0 },
          border: 1,
          borderColor: theme.palette.gray[20],
        })}
        classes={{ root: fluidFontClassName }}
      >
        {inner}
      </Modal>
    );
  },
);

export type TypeFormDefaults = { name: string; description: string };

export type TypeFormProps<T extends TypeFormDefaults = TypeFormDefaults> = {
  onClose?: () => void;
  modalTitle: ReactNode;
  popupState: PopupState;
  onSubmit: (data: T) => Promise<void> | void;
  submitButtonProps: TypeFormSubmitProps;
  disabledFields?: (keyof DeepPartial<T>)[];
  getDefaultValues: () => DefaultValues<T>;
  getDirtyFields?: () => DeepPartial<T>;
};

export const TypeForm = <T extends TypeFormDefaults>({
  children,
  disabledFields = [],
  getDefaultValues,
  getDirtyFields,
  kind,
  onClose,
  modalTitle,
  popupState,
  onSubmit,
  submitButtonProps,
  validateTitle,
}: {
  children?: ReactNode;
  kind: "link" | "property";
  validateTitle: (title: string) => ReturnType<TitleValidationFunction>;
} & TypeFormProps<T>) => {
  const defaultValues = useMemo(() => getDefaultValues(), [getDefaultValues]);
  const defaultDirtyValues = useMemo(
    () => getDirtyFields?.(),
    [getDirtyFields],
  );

  const formMethods = useForm<T>({
    defaultValues,
    shouldFocusError: true,
    mode: "onSubmit",
    reValidateMode: "onSubmit",
  });

  const {
    handleSubmit: wrapHandleSubmit,
    formState: { isSubmitting, isDirty },
    setFocus,
    setValue,
  } = formMethods;

  const lastDefaultDirtyValues = useRef<null | typeof defaultDirtyValues>(null);

  useEffect(() => {
    if (lastDefaultDirtyValues.current === defaultDirtyValues) {
      return;
    }

    lastDefaultDirtyValues.current = defaultDirtyValues;

    if (defaultDirtyValues) {
      for (const [key, value] of Object.entries(defaultDirtyValues)) {
        /**
         * I tried typing Object.entries but it didn't fix the issue – I think
         * this is an issue with react-hook-form's types,not ours.
         */
        setValue(
          // @ts-expect-error key is already equivalent to Path<T> (or would be if using typed entries)
          key,
          value,
          {
            shouldDirty: true,
            shouldTouch: true,
            shouldValidate: true,
          },
        );
      }
    }
  }, [setValue, defaultDirtyValues]);

  const defaultField =
    (defaultDirtyValues?.name ?? defaultValues.name) ? "description" : "name";

  useEffect(() => {
    setFocus(
      // @ts-expect-error trigger expects Path<T>, but key is already equivalent
      defaultField,
    );
  }, [setFocus, defaultField]);

  const handleSubmit = wrapHandleSubmit(onSubmit);

  return (
    <FormProvider {...formMethods}>
      <Box
        sx={(theme) => ({
          px: 2.5,
          pr: 1.5,
          pb: 1.5,
          pt: 2,
          borderBottom: 1,
          borderColor: theme.palette.gray[20],
          alignItems: "center",
          display: "flex",
        })}
      >
        <Typography
          variant="regularTextLabels"
          sx={{ fontWeight: 500, display: "flex", alignItems: "center" }}
        >
          {modalTitle}
        </Typography>
        <IconButton
          {...withHandler(bindToggle(popupState), onClose)}
          sx={(theme) => ({
            ml: "auto",
            svg: {
              color: theme.palette.gray[50],
              fontSize: 20,
            },
          })}
          disabled={isSubmitting}
        >
          <FontAwesomeIcon icon={faClose} />
        </IconButton>
      </Box>
      <Box
        minWidth={{ sm: 500 }}
        p={3}
        component="form"
        display="block"
        onSubmit={(event: FormEvent<HTMLFormElement>) => {
          event.preventDefault();
          event.stopPropagation(); // stop the parent submit being triggered

          void handleSubmit();
        }}
      >
        <Stack
          alignItems="flex-start"
          spacing={3}
          sx={{
            [`.${inputLabelClasses.root}`]: {
              display: "flex",
              alignItems: "center",
            },
          }}
        >
          <TypeFormTitleField
            fieldDisabled={disabledFields.includes("name")}
            validateTitle={validateTitle}
          />
          <TypeFormDescriptionField
            defaultValues={defaultValues}
            kind={kind}
            fieldDisabled={disabledFields.includes("description")}
          />
          {children}
        </Stack>
        <Divider sx={{ mt: 2, mb: 3 }} />
        <Stack direction="row" spacing={1.25}>
          <Button
            {...submitButtonProps}
            loading={isSubmitting}
            disabled={isSubmitting || !isDirty}
            type="submit"
            size="small"
          >
            {submitButtonProps.children}
          </Button>
          <Button
            {...withHandler(bindToggle(popupState), onClose)}
            disabled={isSubmitting}
            size="small"
            variant="tertiary"
          >
            Discard draft
          </Button>
        </Stack>
      </Box>
    </FormProvider>
  );
};
