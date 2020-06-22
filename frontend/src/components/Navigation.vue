<template>
  <div>
    <v-list-item>
      <v-list-item-content>
        <v-list-item-title class="title">
          <router-link
            :to="isAuthenticated ? '/cockpit' : '/'"
            :title="isAuthenticated ? 'Cockpit' : 'Home'"
            style="text-decoration:none"
            class="black--text"
          >Digester</router-link>
        </v-list-item-title>
        <v-list-item-subtitle>End constant interruptions</v-list-item-subtitle>
      </v-list-item-content>
    </v-list-item>
    <v-divider></v-divider>
    <v-list-item link title="Login" v-if="!isAuthenticated" to="/auth/login">
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
    <v-list-item v-if="isAuthenticated" to="/updates">
      <v-list-item-icon>
        <v-icon>{{ updatesIcon }}</v-icon>
      </v-list-item-icon>
      <v-list-item-content>
        <v-list-item-title>Updates</v-list-item-title>
      </v-list-item-content>
    </v-list-item>
    <v-divider></v-divider>
    <v-list-group no-action sub-group v-if="isAuthenticated">
      <template v-slot:activator>
        <v-list-item-title>Advanced</v-list-item-title>
      </template>
      <v-list-item to="/lists">
        <v-list-item-content>
          <v-list-item-title>My Lists</v-list-item-title>
        </v-list-item-content>
        <v-list-item-icon>
          <v-icon>{{ updatesIcon }}</v-icon>
        </v-list-item-icon>
      </v-list-item>
      <v-list-item to="/settings">
        <v-list-item-content>
          <v-list-item-title>Settings</v-list-item-title>
        </v-list-item-content>
        <v-list-item-icon>
          <v-icon>{{ settingsIcon }}</v-icon>
        </v-list-item-icon>
      </v-list-item>
    </v-list-group>
  </div>
</template>

<script>
import {
  mdiAccount,
  mdiSettingsOutline,
  mdiPlaylistCheck,
  mdiFormatListBulletedTriangle
} from "@mdi/js";

export default {
  data() {
    return {
      accountIcon: mdiAccount,
      settingsIcon: mdiSettingsOutline,
      subsIcon: mdiPlaylistCheck,
      updatesIcon: mdiFormatListBulletedTriangle
    };
  },

  computed: {
    isAuthenticated() {
      return this.$store.getters.isAuthenticated;
    }
  }
};
</script>
