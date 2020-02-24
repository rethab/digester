<template>
  <v-container>
    <v-dialog :value="true" fullscreen hide-overlay>
      <DialogToolbar v-on:closeDialog="$emit('closeDialog')" />
      <v-card>
        <v-card-title>Edit {{list.name}}</v-card-title>
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
              <v-btn type="submit" @click="submit" class="primary">Search</v-btn>
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
        <div v-if="channels.length > 0">
          <v-card-title>Channels in this List ({{channels.length}})</v-card-title>
          <v-card-text>
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
                  <v-btn text icon color="error">
                    <v-icon>{{removeIcon}}</v-icon>
                  </v-btn>
                </v-list-item-action>
              </v-list-item>
            </v-list>
          </v-card-text>
        </div>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import ChannelInput from "@/components/channels/ChannelInput.vue";
import ChannelSearchResults from "@/components/channels/ChannelSearchResults.vue";
import DialogToolbar from "@/components/common/DialogToolbar.vue";
import Channel from "@/models/Channel.js";
import Api from "@/services/api.js";
import { mdiMinusCircle } from "@mdi/js";
export default {
  components: {
    ChannelInput,
    ChannelIcon,
    DialogToolbar,
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
      removeIcon: mdiMinusCircle,

      channel: new Channel("RssFeed", null),
      nameErrors: [],
      channels: this.list.channels,

      searchResults: null,
      searchError: null
    };
  },
  computed: {},
  methods: {
    submit() {
      let params = {
        channel_type: this.channel.type,
        query: this.channel.name
      };
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
        });
    },
    addChannel(channel) {
      this.$store
        .dispatch("addChannel", {
          list: this.list,
          channel: channel
        })
        .then(() => {
          this.channels.push(channel);
        });
    },
    alreadyInList(channel) {
      return this.channels.some(chan => chan.id == channel.id);
    }
  }
};
</script>
