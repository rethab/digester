<template>
  <v-footer app absolute>
    <router-link to="/privacy" style="text-decoration: none; color: inherit;">Privacy</router-link>
    <v-spacer></v-spacer>
    <v-btn href="https://twitter.com/digesterapp" target="_blank" icon>
      <v-icon>{{twitterIcon}}</v-icon>
    </v-btn>
    <v-btn href="https://linkedin.com/company/digester" target="_blank" icon>
      <v-icon>{{linkedinIcon}}</v-icon>
    </v-btn>
    <v-btn href="https://www.youtube.com/channel/UCWDpenaoBUsi0TmFiOK611A" target="_blank" icon>
      <v-icon>{{youtubeIcon}}</v-icon>
    </v-btn>
  </v-footer>
</template>

<script>
import { mdiTwitter, mdiYoutube, mdiLinkedin } from "@mdi/js";
export default {
  data() {
    return {
      twitterIcon: mdiTwitter,
      youtubeIcon: mdiYoutube,
      linkedinIcon: mdiLinkedin
    };
  }
};
</script>