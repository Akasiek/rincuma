<script setup lang="ts">
import { Field } from "vee-validate";
import Input from "./Input.vue";
import Textarea from "./Textarea.vue";
import AppSelect from "../AppSelect.vue";
import type { SelectOption } from "#shared/types/select";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    name: string;
    label: string;
    id?: string;
    as?: "input" | "select" | "textarea";
    options?: SelectOption[];
    description?: string;
    optional?: boolean;
    validateOnBlur?: boolean;
    validateOnChange?: boolean;
    validateOnInput?: boolean;
    validateOnModelUpdate?: boolean;
  }>(),
  {
    as: "input",
    options: () => [],
    validateOnBlur: true,
    validateOnChange: true,
    validateOnInput: false,
    validateOnModelUpdate: true,
  },
);

const fieldId = computed(() => props.id ?? props.name);
const textControl = computed(() => (props.as === "textarea" ? Textarea : Input));
</script>

<template>
  <Field
    v-slot="{ componentField, errorMessage, handleChange, handleBlur }"
    :name="name"
    :label="label"
    :validate-on-blur="validateOnBlur"
    :validate-on-change="validateOnChange"
    :validate-on-input="validateOnInput"
    :validate-on-model-update="validateOnModelUpdate"
  >
    <FormGroup
      :id="fieldId"
      :label="label"
      :description="description"
      :optional="optional"
      :error="errorMessage"
    >
      <template #actions><slot name="actions" /></template>
      <template #default="{ describedby, invalid }">
        <AppSelect
          v-if="as === 'select'"
          v-bind="$attrs"
          :id="fieldId"
          :label="label"
          :model-value="componentField.modelValue ?? null"
          :options="options"
          :invalid="invalid"
          :describedby="describedby"
          hide-label
          @update:model-value="handleChange($event, validateOnModelUpdate || validateOnChange)"
          @blur="handleBlur($event, validateOnBlur)"
        />
        <component
          :is="textControl"
          v-else
          v-bind="{ ...$attrs, ...componentField }"
          :id="fieldId"
          :invalid="invalid"
          :aria-describedby="describedby"
        >
          <slot />
        </component>
      </template>
    </FormGroup>
  </Field>
</template>
