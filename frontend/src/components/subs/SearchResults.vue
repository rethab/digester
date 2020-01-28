<template>
  <div>
    <div class="text-center">
      <h1 class="title">Found {{channels.length}} Result(s)</h1>
    </div>
    <v-card v-if="channels.length == 0" color="error" class="lighten-3">
      <v-card-text v-html="noResultsText" />
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn @click="$vuetify.goTo('#searchInput')" outlined>Try another?</v-btn>
      </v-card-actions>
    </v-card>
    <v-row v-else>
      <v-col
        v-for="(channel, i) in channels"
        :key="i+'-'+channel.id"
        cols="12"
        :md="channels.length == 1 ? 12 : 6"
      >
        <v-card color="secondary" class="lighten-4" raised>
          <v-card-title>
            <span
              :class="alreadySubscribed(channel) ? 'grey--text' : ''"
              style="word-break: break-word"
            >
              <ChannelIcon :type="channel.channel_type" />
              {{channel.name}}
            </span>
          </v-card-title>
          <v-card-subtitle>
            <a
              class="grey--text mt-n3"
              :href="channel.link"
              style="text-decoration: none"
              target="_blank"
            >{{channel.link}}</a>
          </v-card-subtitle>
          <v-card-actions class="mt-n5">
            <v-spacer></v-spacer>
            <v-btn v-if="alreadySubscribed(channel)" text disabled>Already subscribed</v-btn>
            <v-btn v-else @click="$emit('openDialog', channel)" fab dark small class="primary">
              <v-icon dark>{{plusIcon}}</v-icon>
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import { mdiPlus } from "@mdi/js";
import { mapGetters } from "vuex";

export default {
  components: {
    ChannelIcon
  },
  props: {
    channelType: {
      type: String,
      required: true
    },
    channels: {
      type: Array,
      required: true
    }
  },
  data() {
    return {
      plusIcon: mdiPlus
    };
  },
  computed: {
    ...mapGetters(["subscriptions"]),
    noResultsText() {
      if (this.channelType == "RssFeed") {
        return "We could not find a blog with this URL.<br /><br />Are you sure this points to a blog? (We need an RSS or Atom feed). If you think this should be working, please contact us at info@digester.app and we'll take a look.";
      } else if (this.channelType == "GithubRelease") {
        return "We could not find a repository with this name.<br /><br />Are you sure it exists and is public?";
      } else {
        return "Found no results";
      }
    }
  },
  methods: {
    alreadySubscribed(channel) {
      return this.subscriptions.some(sub => sub.channelId == channel.id);
    }
  }
};
</script>
