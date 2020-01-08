<template>
  <v-snackbar v-model="snackbar" :timeout="10000" :top="true">
    Either you are offline or we are offline. ¯\_(ツ)_/¯
    <v-btn text @click="snackbar = false" color="error">Close</v-btn>
  </v-snackbar>
</template>

<script>
import { mapGetters } from "vuex";
export default {
  data() {
    return {
      snackbar: false
    };
  },
  computed: {
    ...mapGetters({ isOffline: "isOffline" })
  },

  watch: {
    isOffline: "updateSnackbar"
  },

  methods: {
    updateSnackbar(value) {
      if (value) {
        this.snackbar = true;
        // we directly set it back to online, because
        // otherwise we wouldn't be triggered on the watch
        // anymore if the user does something else, because
        // we would only set offline=true while it is already
        // true, which means no update is fired.
        this.$store.dispatch("setOnline");
      }
    }
  }
};
</script>