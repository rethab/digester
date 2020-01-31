<template>
  <v-snackbar v-model="snackbar" :timeout="10000" :top="true">
    {{ message() }}
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
    },
    message() {
      if (window.navigator && !window.navigator.onLine) {
        return "Are you connected to the internet?";
      } else {
        return "Looks like our hamsters are having some trouble keeping up. Please give them some rest and try again.";
      }
    }
  }
};
</script>