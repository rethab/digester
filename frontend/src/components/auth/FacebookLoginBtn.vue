<template>
  <div style="display:inline">
    <v-btn
      color="#3B5998"
      class="white--text"
      :loading="loading"
      :disabled="disabled"
      @click="$emit('authenticate')"
    >
      Facebook
      <v-icon small class="pl-1">{{ facebookIcon }}</v-icon>
    </v-btn>
  </div>
</template>

<script>
import { mdiFacebook } from "@mdi/js";
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
      facebookIcon: mdiFacebook
    };
  }
};
</script>