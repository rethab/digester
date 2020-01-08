<template>
  <v-btn dark :loading="loading" @click="authenticate('github')">
    Github
    <v-icon small class="pl-1">{{ githubIcon }}</v-icon>
  </v-btn>
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
    authenticate(provider) {
      this.loading = true;
      this.$store
        .dispatch("authenticate", {
          vueAuth: this.$auth,
          provider: provider
        })
        .then(resp => {
          this.loading = false;
          if (resp.data.first_login) {
            this.$router.push({ name: "home", query: { firstLogin: true } });
          } else {
            this.$router.push({ name: "subscriptions" });
          }
        })
        .catch(err => {
          if (!err.response) {
            this.$store.dispatch("setOffline");
          }
          this.loading = false;
        });
    }
  }
};
</script>