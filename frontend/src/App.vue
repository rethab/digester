<template>
  <v-app>
    <v-app-bar app color="primary" dark>
      <v-toolbar-title class="headline text-uppercase">
        <v-btn text to="/">Digester</v-btn>
      </v-toolbar-title>
      <v-btn v-if="isAuthenticated" text to="/blogs/add">Add</v-btn>
      <v-btn v-if="!isAuthenticated" text to="/auth/login">Login</v-btn>
      <v-btn v-if="isAuthenticated" text to="/auth/logout">Logout</v-btn>
      <v-spacer></v-spacer>
    </v-app-bar>

    <v-content>
      <router-view />
    </v-content>
  </v-app>
</template>

<script>
export default {
  name: "App",

  computed: {
    isAuthenticated() {
      return this.$store.getters.isAuthenticated;
    }
  },

  created() {
    document.title = process.env.VUE_APP_TITLE;
  }
};
</script>
