<template>
  <div style="display:inline">
    <v-btn
      color="black"
      class="white--text"
      :loading="loading"
      :disabled="disabled"
      @click="$emit('authenticate')"
    >
      Github
      <v-icon small class="pl-1">{{ githubIcon }}</v-icon>
    </v-btn>
  </div>
</template>

<script>
import { mdiGithubCircle } from "@mdi/js";
export default {
  props: {
    disabled: {
      type: Boolean,
      default: false
    },
    loading: {
      type: Boolean,
      required: true
    }
  },
  data() {
    return {
      githubIcon: mdiGithubCircle
    };
  }
};
</script>