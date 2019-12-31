<template>
  <v-container>
    <v-card color="error">
      <v-card-title>{{title}}</v-card-title>
      <v-card-text>
        <span v-html="message"></span>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn outlined :to="to">Fix this!</v-btn>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script>
export default {
  props: {
    title: {
      type: String,
      required: true
    },
    message: {
      type: String,
      required: true
    },
    to: {
      type: String,
      required: true
    }
  }
};
</script>