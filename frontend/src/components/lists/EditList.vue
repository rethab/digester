<template>
  <div>
    <v-card flat>
      <v-card-title>{{list.name}}</v-card-title>
      <v-card-subtitle>Add Channels to Your List</v-card-subtitle>
      <v-form @submit.prevent="submit">
        <v-card-text>
          <section id="searchInput">
            <ChannelInput
              v-on:selectChannel="channel = $event"
              v-bind:value="channel"
              :nameErrors="nameErrors"
            />
          </section>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn type="submit" :loading="loading" class="primary">Search</v-btn>
          </v-card-actions>
        </v-card-text>
      </v-form>
      <section id="searchResults">
        <ChannelSearchResults
          class="mt-6 px-5"
          v-if="searchResults"
          v-on:channelSelected="addChannel"
          :channels="searchResults"
          :channelType="channel.type"
          :searchError="searchError"
          :alreadyThere="alreadyInList"
          alreadyThereMessage="Already in List"
        />
      </section>
      <v-card-title>
        <span v-if="channels.length == 0">No channels in this List</span>
        <span v-else-if="channels.length == 1">One channel in this List</span>
        <span v-else>{{channels.length}} channels in this List</span>
      </v-card-title>
      <v-card-text v-if="channels.length > 0">
        <v-list>
          <v-list-item v-for="(channel, idx) in channels" :key="idx" class="pl-0" color="red">
            <v-list-item-avatar>
              <ChannelIcon :type="channel.type" />
            </v-list-item-avatar>
            <v-list-item-content>
              <v-list-item-title>{{channel.name}}</v-list-item-title>
              <v-list-item-subtitle>
                <a
                  :href="channel.link"
                  target="_blank"
                  style="text-decoration: none"
                >{{channel.link}}</a>
              </v-list-item-subtitle>
            </v-list-item-content>
            <v-list-item-action>
              <v-btn @click="removeChannel(channel)" text icon>
                <v-icon color="error lighten-1">{{removeIcon}}</v-icon>
              </v-btn>
            </v-list-item-action>
          </v-list-item>
        </v-list>
      </v-card-text>
    </v-card>
  </div>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import ChannelInput from "@/components/channels/ChannelInput.vue";
import ChannelSearchResults from "@/components/channels/ChannelSearchResults.vue";
import Channel from "@/models/Channel.js";
import Api from "@/services/api.js";
import { mdiDelete } from "@mdi/js";
export default {
  components: {
    ChannelInput,
    ChannelIcon,
    ChannelSearchResults
  },
  props: {
    list: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      removeIcon: mdiDelete,

      channel: new Channel("RssFeed", null),
      nameErrors: [],

      searchResults: null,
      searchError: null,
      loading: false
    };
  },
  computed: {
    channels() {
      return this.$store.getters.channelsByListId(this.list.id);
    }
  },
  methods: {
    submit() {
      this.searchResults = null;
      this.searchError = null;
      this.nameErrors = [];

      let params = {
        channel_type: this.channel.type,
        query: this.channel.name
      };

      let nameErrors = this.channel.validate();
      if (nameErrors.length != 0) {
        this.nameErrors = nameErrors;
        return;
      }

      this.loading = true;

      Api()
        .get("subscriptions/search", {
          params: params,
          timeout: 6000 // default timeout is 4s, but here we need to query upstream..
        })
        .then(resp => {
          this.loading = false;

          // shortcut
          const oneNewResult =
            resp.data.channels.length === 1 &&
            !this.alreadyInList(resp.data.channels[0]);

          if (oneNewResult) {
            this.addChannel(resp.data.channels[0]);
          } else {
            this.searchResults = resp.data.channels;
          }
        })
        .catch(err => {
          this.loading = false;
          if (err.response && err.response.data && err.response.data.error) {
            this.searchError = err.response.data.error;
          } else {
            this.searchError =
              "Something went wrong. Please make sure the input is valid. Contact us info@digester.app if you think this should be working.";
          }
        });
    },
    addChannel(channel) {
      this.$store.dispatch("addChannel", {
        list: this.list,
        channel: channel
      });
      // fixme handle error
    },
    removeChannel(channel) {
      this.$store.dispatch("removeChannel", {
        list: this.list,
        channel: channel
      });
      // fixme handle error
    },
    alreadyInList(channel) {
      return this.channels.some(chan => chan.id == channel.id);
    }
  }
};
</script>
