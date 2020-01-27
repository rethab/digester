<template>
  <div>
    <v-subheader :key="index+'sh'" style="height: 20px">
      <v-icon small class="mr-1">{{githubIcon}}</v-icon>
      <a
        :href="value.channelName | urlify"
        target="_blank"
        style="text-decoration: none"
        class="mr-1"
        color="secondary"
      >{{value.channelName}}</a>
      |
      <span v-if="!mobile" class="ml-1">{{value.title}} |</span>
      <span class="ml-1">{{ value.published | formatDate }}</span>
    </v-subheader>
    <v-divider :key="index + 'd'"></v-divider>
    <v-list-item :key="index + 'li'" class="mb-2" style="min-height: 25px">
      <v-list-item-title>
        <a
          :href="value.url"
          target="_blank"
          class="black--text"
          style="font-size: 1.1em"
        >{{linkText}}</a>
      </v-list-item-title>
    </v-list-item>
  </div>
</template>

<script>
import { mdiGithubCircle } from "@mdi/js";
import moment from "moment";
export default {
  components: {},
  props: {
    value: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      githubIcon: mdiGithubCircle,

      mobile: this.$vuetify.breakpoint.smAndDown
    };
  },
  computed: {
    linkText() {
      if (this.mobile && this.value.title) return this.value.title;
      else return this.value.url;
    }
  },
  filters: {
    urlify(repository) {
      return `https://github.com/${repository}`;
    },
    formatDate(datetime) {
      return moment(datetime, "YYYY-MM-DDTHH:mm:ss").calendar(null, {
        lastWeek: "dddd",
        lastDay: "[Yesterday at] HH:mm",
        sameDay: "[Today at] HH:mm"
      });
    }
  }
};
</script>