<template>
  <div>
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
              <v-btn @click="submit" class="primary">Search</v-btn>
            </v-card-actions>
          </v-card-text>
        </v-form>
        <section id="searchResults">
          <ChannelSearchResults
            class="mt-6"
            v-if="searchResults"
            v-on:channelSelected="addChannel"
            :channels="searchResults"
            :channelType="channel.type"
            :searchError="searchError"
            :alreadyThere="alreadyInList"
            alreadyThereMessage="Already in List"
          />
        </section>
        <div>
          <v-card-title>Channels in this List</v-card-title>
          <v-card-text>
            <ul>
              <li v-for="(channel, idx) in channels" :key="idx">{{channel.name}}</li>
            </ul>
          </v-card-text>
        </div>
      </v-card>
    </v-dialog>
  </div>
</template>

<script>
import ChannelInput from "@/components/channels/ChannelInput.vue";
import ChannelSearchResults from "@/components/channels/ChannelSearchResults.vue";
import DialogToolbar from "@/components/common/DialogToolbar.vue";
import Channel from "@/models/Channel.js";
import Api from "@/services/api.js";
export default {
  components: {
    ChannelInput,
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
          this.searchResults = resp.data.channels;
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
