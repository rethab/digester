<template>
  <v-btn dark @click="authenticate('github')">
    Github
    <v-icon small class="pl-1">mdi-github-circle</v-icon>
  </v-btn>
</template>

<script>
export default {
  methods: {
    authenticate(provider) {
      this.$store
        .dispatch("authenticate", {
          vueAuth: this.$auth,
          provider: provider
        })
        .then(resp => {
          if (resp.data.first_login) {
            this.$router.push({ name: "home", query: { firstLogin: true } });
          } else {
            this.$router.push({ name: "subscriptions" });
          }
        })
        .catch(() => {});
    }
  }
};
</script>