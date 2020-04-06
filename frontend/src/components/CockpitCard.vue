<template>
  <v-card :to="link" color="secondary" class="lighten-4" height="190px">
    <v-card-title>{{title}}</v-card-title>
    <v-card-subtitle class="text--secondary">..{{subtitle}}</v-card-subtitle>
    <v-card-text class="text--primary">
      <div style="height:40px;">{{text}}</div>
    </v-card-text>
    <v-card-actions>
      <v-spacer></v-spacer>
      <v-btn :to="link" text>open</v-btn>
    </v-card-actions>
  </v-card>
</template>

<script>
export default {
  props: {
    title: {
      type: String,
      required: true
    },
    subtitle: {
      type: String,
      required: true
    },
    text: {
      type: String,
      required: true
    },
    link: {
      type: String,
      required: true
    }
  }
};
</script>