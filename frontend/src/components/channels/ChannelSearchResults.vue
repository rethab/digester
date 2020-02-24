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
    <v-list v-else>
      <v-list-item v-for="(channel, i) in channels" :key="i" class="px-0">
        <v-list-item-avatar>
          <ChannelIcon :type="channel.type" />
        </v-list-item-avatar>
        <v-list-item-content>
          <v-list-item-title style="word-break: break-word">
            {{channel.name}}
            <span
              v-if="alreadyThere(channel)"
              class="font-italic grey--text caption"
            >&nbsp;({{alreadyThereMessage}})</span>
          </v-list-item-title>
          <v-list-item-subtitle>
            <a
              class="grey--text mt-n3"
              :href="channel.link"
              style="text-decoration: none"
              target="_blank"
            >{{channel.link}}</a>
          </v-list-item-subtitle>
        </v-list-item-content>
        <v-list-item-action>
          <v-btn
            :disabled="alreadyThere(channel)"
            @click="$emit('channelSelected', channel)"
            icon
            :dark="!alreadyThere(channel)"
            small
            :class="alreadyThere(channel) ? '' : 'primary'"
          >
            <v-icon>{{plusIcon}}</v-icon>
          </v-btn>
        </v-list-item-action>
      </v-list-item>
    </v-list>
  </div>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import { mdiPlus } from "@mdi/js";

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
    },
    searchError: {
      type: String
    },
    alreadyThere: {
      type: Function,
      required: true
    },
    alreadyThereMessage: {
      type: String,
      required: true
    }
  },
  data() {
    return {
      plusIcon: mdiPlus
    };
  },
  computed: {
    noResultsText() {
      // specific error message from server or generic 'no results' message
      // based on channel type
      if (this.searchError) {
        return this.searchError;
      } else if (this.channelType == "RssFeed") {
        return "We could not find a blog with this URL.<br /><br />Are you sure this points to a blog? (We need an RSS or Atom feed). If you think this should be working, please contact us at info@digester.app and we'll take a look.";
      } else if (this.channelType == "GithubRelease") {
        return "We could not find a repository with this name.<br /><br />Are you sure it exists and is public?";
      } else {
        return "Found no results";
      }
    }
  }
};
</script>
