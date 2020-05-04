<template>
  <v-app>
    <v-navigation-drawer v-model="drawer" app>
      <Navigation />
      <template v-if="isAuthenticated" v-slot:append>
        <div class="pa-2">
          <LogoutBtn />
        </div>
      </template>
    </v-navigation-drawer>

    <v-app-bar app color="primary" dark>
      <v-app-bar-nav-icon @click.stop="drawer = !drawer"></v-app-bar-nav-icon>
      <v-toolbar-title>
        <router-link
          :to="isAuthenticated ? '/cockpit' : '/'"
          class="white--text"
          style="text-decoration:none"
        >Digester</router-link>
      </v-toolbar-title>
      <v-spacer></v-spacer>
      <div>
        <router-link
          v-if="isAuthenticated"
          to="/settings"
          class="white--text"
          style="text-decoration: none"
        >
          <v-icon>{{ accountIcon }}</v-icon>
          {{ username }}
        </router-link>
        <router-link v-else to="/auth/login" class="white--text" style="text-decoration: none">Login</router-link>
      </div>
    </v-app-bar>

    <OfflineSnackbar />

    <v-content>
      <v-container fluid>
        <router-view />
      </v-container>
    </v-content>
    <Footer />
  </v-app>
</template>

<script>
import { mdiAccount } from "@mdi/js";
import Navigation from "@/components/Navigation.vue";
import Footer from "@/components/Footer.vue";
import OfflineSnackbar from "@/components/common/OfflineSnackbar.vue";
import LogoutBtn from "@/components/auth/LogoutBtn.vue";

export default {
  name: "App",

  components: {
    Navigation,
    Footer,
    OfflineSnackbar,
    LogoutBtn
  },

  created() {
    document.title = process.env.VUE_APP_TITLE;
  },

  data() {
    return {
      drawer: null,

      accountIcon: mdiAccount
    };
  },

  computed: {
    isAuthenticated() {
      return this.$store.getters.isAuthenticated;
    },
    username() {
      return this.$store.getters.username;
    }
  }
};
</script>
