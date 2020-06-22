<template>
  <v-footer app absolute class="grey--text">
    <v-container class="pa-0">
      <v-row class="ma-0" align="center">
        <v-col class="pa-0 overline font-weight-thin">
          <router-link
            to="/privacy"
            title="Privacy Policy"
            style="text-decoration: none; color: grey;"
          >Privacy</router-link>
          <span class="mx-1">|</span>
          <router-link
            to="/terms"
            title="Terms & Conditions"
            style="text-decoration: none; color: grey;"
          >Terms</router-link>
        </v-col>
        <v-col class="text-center pa-0" v-if="$vuetify.breakpoint.smAndUp">Digester &copy; 2020</v-col>
        <v-col class="text-right pa-0">
          <v-btn
            v-for="(social, idx) in socialLinks"
            :key="idx"
            :href="social.url"
            :title="`Follow Digester on ${social.title}`"
            icon
            color="grey"
            class="ml-2 text--lighten-1"
            target="_blank"
            small
          >
            <v-icon>{{social.icon}}</v-icon>
          </v-btn>
        </v-col>
      </v-row>
    </v-container>
  </v-footer>
</template>

<script>
import { mdiTwitter, mdiYoutube, mdiLinkedin } from "@mdi/js";
export default {
  data() {
    return {
      socialLinks: [
        {
          url: "https://twitter.com/digesterapp",
          title: "Twitter",
          icon: mdiTwitter
        },
        {
          url: "https://linkedin.com/company/digester",
          title: "Linkedin",
          icon: mdiLinkedin
        },
        {
          url: "https://www.youtube.com/channel/UCWDpenaoBUsi0TmFiOK611A",
          title: "Youtube",
          icon: mdiYoutube
        }
      ]
    };
  }
};
</script>