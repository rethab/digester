<template>
  <v-icon :small="small">{{icon}}</v-icon>
</template>
<script>
import {
  mdiRss,
  mdiTwitter,
  mdiGithubCircle,
  mdiFormatListBulletedTriangle
} from "@mdi/js";
export default {
  props: {
    type: {
      type: String,
      required: true
    },
    small: {
      type: Boolean,
      default: false
    }
  },
  data() {
    return {
      githubIcon: mdiGithubCircle,
      rssIcon: mdiRss,
      twitterIcon: mdiTwitter,
      listIcon: mdiFormatListBulletedTriangle
    };
  },
  computed: {
    icon() {
      switch (this.type) {
        case "GithubRelease":
          return this.githubIcon;
        case "RssFeed":
          return this.rssIcon;
        case "Twitter":
          return this.twitterIcon;
        case "List":
          return this.listIcon;
        default:
          return "";
      }
    }
  }
};
</script>