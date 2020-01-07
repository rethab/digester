<template>
  <v-app>
    <v-navigation-drawer v-model="drawer" app temporary>
      <v-list-item>
        <v-list-item-content>
          <v-list-item-title class="title">
            <router-link to="/" style="text-decoration:none" class="black--text">Digester</router-link>
          </v-list-item-title>
          <v-list-item-subtitle>End constant interruptions</v-list-item-subtitle>
        </v-list-item-content>
      </v-list-item>
      <v-divider></v-divider>
      <v-list-item link v-if="!isAuthenticated" to="/auth/login">
        <v-list-item-icon>
          <v-icon>mdi-account</v-icon>
        </v-list-item-icon>
        <v-list-item-content>
          <v-list-item-title>Login</v-list-item-title>
        </v-list-item-content>
      </v-list-item>
      <v-list-item v-if="isAuthenticated" to="/subs">
        <v-list-item-icon>
          <v-icon>mdi-playlist-check</v-icon>
        </v-list-item-icon>
        <v-list-item-content>
          <v-list-item-title>Subscriptions</v-list-item-title>
        </v-list-item-content>
      </v-list-item>
      <v-list-item v-if="isAuthenticated" to="/settings">
        <v-list-item-icon>
          <v-icon>mdi-settings-outline</v-icon>
        </v-list-item-icon>
        <v-list-item-content>
          <v-list-item-title>Settings</v-list-item-title>
        </v-list-item-content>
      </v-list-item>
      <template v-if="isAuthenticated" v-slot:append>
        <div class="pa-2">
          <v-btn dark block to="/auth/logout">Logout</v-btn>
        </div>
      </template>
    </v-navigation-drawer>

    <v-app-bar app color="primary" dark>
      <v-app-bar-nav-icon @click.stop="drawer = !drawer"></v-app-bar-nav-icon>
      <v-toolbar-title>
        <router-link to="/" class="white--text" style="text-decoration:none">Digester</router-link>
      </v-toolbar-title>
    </v-app-bar>

    <v-content>
      <v-container fluid>
        <router-view />
      </v-container>
    </v-content>
  </v-app>
</template>

<script>
export default {
  name: "App",

  data() {
    return {
      drawer: null
    };
  },

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
