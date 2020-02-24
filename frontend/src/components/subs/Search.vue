<template>
  <v-card>
    <v-card-title>New Subscription</v-card-title>
    <v-card-subtitle v-if="channel.isGithubRelease()">
      Into golang? Try
      <span class="font-italic">golang/tools</span>
    </v-card-subtitle>
    <v-card-subtitle v-if="channel.isRss()">
      Into tech news? Try
      <span class="font-italic">theverge.com</span>
    </v-card-subtitle>
    <v-form @submit.prevent="submit">
      <v-card-text class="pb-0">
        <ChannelInput
          v-on:selectChannel="channel = $event"
          v-bind:value="channel"
          :nameErrors="nameErrors"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn type="submit" :loading="loading" class="primary">Search</v-btn>
      </v-card-actions>
    </v-form>
  </v-card>
</template>

<script>
import ChannelInput from "@/components/channels/ChannelInput.vue";
import Channel from "@/models/Channel.js";
export default {
  components: {
    ChannelInput
  },
  props: {
    loading: {
      type: Boolean,
      required: true
    },
    initialValue: {
      type: String,
      required: true
    }
  },
  data() {
    return {
      snackbar: false,

      channel: new Channel("RssFeed", this.initialValue),

      nameErrors: [],

      searchResults: null,
      selectedChannel: null,
      dialog: null
    };
  },
  computed: {
    hasErrors() {
      return this.nameErrors.length == 0;
    }
  },
  watch: {
    initialValue(newValue) {
      this.channel.name = newValue;
    }
  },
  methods: {
    submit() {
      this.clearErrors();
      if (this.validate()) {
        console.log(`Search.search(${this.channel.show()})`);
        this.$emit("search", this.channel.type, this.channel.name);
      }
    },
    validate() {
      this.nameErrors = this.channel.validate();
      return this.hasErrors;
    },
    clearErrors() {
      this.nameErrors = [];
    }
  }
};
</script>