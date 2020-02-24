<template>
  <v-toolbar dark color="primary">
    <v-btn icon dark @click="$emit('closeDialog')">
      <v-icon>{{ closeIcon }}</v-icon>
    </v-btn>
    <v-spacer></v-spacer>
    <v-toolbar-items v-if="actionText">
      <v-btn dark text @click="$emit('dialogAction')">{{actionText}}</v-btn>
    </v-toolbar-items>
  </v-toolbar>
</template>

<script>
import { mdiClose } from "@mdi/js";
export default {
  props: {
    actionText: {
      type: String
    }
  },
  data() {
    return {
      closeIcon: mdiClose
    };
  }
};
</script>