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
          <v-icon>{{ accountIcon }}</v-icon>
        </v-list-item-icon>
        <v-list-item-content>
          <v-list-item-title>Login</v-list-item-title>
        </v-list-item-content>
      </v-list-item>
      <v-list-item v-if="isAuthenticated" to="/subs">
        <v-list-item-icon>
          <v-icon>{{ subsIcon }}</v-icon>
        </v-list-item-icon>
        <v-list-item-content>
          <v-list-item-title>Subscriptions</v-list-item-title>
        </v-list-item-content>
      </v-list-item>
      <v-list-item v-if="isAuthenticated" to="/settings">
        <v-list-item-icon>
          <v-icon>{{ settingsIcon }}</v-icon>
        </v-list-item-icon>
        <v-list-item-content>
          <v-list-item-title>Settings</v-list-item-title>
        </v-list-item-content>
      </v-list-item>
      <template v-if="isAuthenticated" v-slot:append>
        <div class="pa-2">
          <LogoutBtn />
        </div>
      </template>
    </v-navigation-drawer>

    <v-app-bar app color="primary" dark>
      <v-app-bar-nav-icon @click.stop="drawer = !drawer"></v-app-bar-nav-icon>
      <v-toolbar-title>
        <router-link to="/" class="white--text" style="text-decoration:none">Digester</router-link>
      </v-toolbar-title>
      <v-spacer></v-spacer>
      <div v-if="isAuthenticated">
        <v-icon>{{ accountIcon }}</v-icon>
        {{ username }}
      </div>
    </v-app-bar>

    <OfflineSnackbar />

    <v-content>
      <v-container fluid>
        <router-view />
      </v-container>
    </v-content>
  </v-app>
</template>

<script>
import { mdiAccount, mdiSettingsOutline, mdiPlaylistCheck } from "@mdi/js";
import OfflineSnackbar from "@/components/common/OfflineSnackbar.vue";
import LogoutBtn from "@/components/auth/LogoutBtn.vue";

export default {
  name: "App",

  components: {
    OfflineSnackbar,
    LogoutBtn
  },

  created() {
    document.title = process.env.VUE_APP_TITLE;
  },

  data() {
    return {
      drawer: null,

      accountIcon: mdiAccount,
      settingsIcon: mdiSettingsOutline,
      subsIcon: mdiPlaylistCheck
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
