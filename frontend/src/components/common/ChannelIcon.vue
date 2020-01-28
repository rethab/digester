<template>
  <v-icon :small="small">{{icon}}</v-icon>
</template>
<script>
import { mdiRss, mdiGithubCircle } from "@mdi/js";
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
      rssIcon: mdiRss
    };
  },
  computed: {
    icon() {
      switch (this.type) {
        case "GithubRelease":
          return this.githubIcon;
        case "RssFeed":
          return this.rssIcon;
        default:
          return "";
      }
    }
  }
};
</script>