<template>
  <v-btn dark block :loading="loading" @click="logout()">Logout</v-btn>
</template>

<script>
import { mdiGithubCircle } from "@mdi/js";
export default {
  data() {
    return {
      loading: false,
      githubIcon: mdiGithubCircle
    };
  },
  methods: {
    logout() {
      this.loading = true;
      this.$store
        .dispatch("unauthenticate")
        .then(() => {
          this.loading = false;
          this.$router.push({ name: "home" });
        })
        .catch(() => {
          this.loading = false;
        });
    }
  }
};
</script>