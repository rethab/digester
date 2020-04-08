<template>
  <v-snackbar v-model="value" :top="true">
    {{message}}
    <v-btn text color="error" @click="$emit('input', false)">Close</v-btn>
  </v-snackbar>
</template>

<script>
export default {
  props: {
    message: {
      type: String,
      required: true
    },
    value: {
      show: {
        required: true
      }
    }
  }
};
</script>