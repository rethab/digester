<template>
  <v-card>
    <v-card-title>New Subscription</v-card-title>
    <v-form @submit.prevent="submit">
      <v-card-text class="py-0">
        <ChannelInput
          v-on:selectChannel="channel = $event"
          v-bind:value="channel"
          :nameErrors="nameErrors"
          :showList="true"
        />
      </v-card-text>
      <v-card-actions>
        <v-dialog v-model="helpDialog" max-width="500" scrollable>
          <template v-slot:activator="{ on }">
            <span v-if="$vuetify.breakpoint.smAndUp" v-on="on" class="grey--text mr-1">Need Help?</span>
            <v-icon color="grey" v-on="on">{{helpIcon}}</v-icon>
          </template>
          <v-card>
            <v-card-title>How to Search</v-card-title>
            <v-card-text class="pb-0">
              <p>
                <span class="font-weight-bold subtitle-1">Twitter:</span>
                Search by handle (eg. @BillGates) or name (eg. Lady Gaga) and receive updates on
                <span
                  class="font-weight-bold"
                >new tweets</span>.
              </p>
              <p>
                <span class="font-weight-bold subtitle-1">Blog/Rss:</span>
                Search by website link (eg. nytimes.com or dilbert.com) and receive updates on
                <strong>new posts</strong>.
              </p>
              <p>
                <span class="font-weight-bold subtitle-1">Github:</span>
                Search by repository (eg. kubernetes/kubernetes) and receive updates on
                <strong>new releases</strong>.
              </p>

              <p>
                <span class="font-weight-bold subtitle-1">List:</span>
                Search by name (eg. Scala) and receive updates for
                <strong>any of the channels</strong> therein.
              </p>
            </v-card-text>
            <v-card-actions>
              <v-spacer></v-spacer>
              <v-btn text @click="helpDialog = false">Close</v-btn>
            </v-card-actions>
          </v-card>
        </v-dialog>

        <v-spacer></v-spacer>
        <v-btn type="submit" :loading="loading" class="primary">Search</v-btn>
      </v-card-actions>
    </v-form>
  </v-card>
</template>

<script>
import ChannelInput from "@/components/channels/ChannelInput.vue";
import Channel from "@/models/Channel.js";
import preferences from "@/services/preferences.js";
import { mdiHelpCircleOutline } from "@mdi/js";
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
      helpIcon: mdiHelpCircleOutline,
      helpDialog: false,

      snackbar: false,

      channel: new Channel(
        preferences().channelType || "Twitter",
        this.initialValue
      ),

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
        preferences().channelType = this.channel.type;
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