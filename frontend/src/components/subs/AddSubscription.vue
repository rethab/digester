<template>
  <v-container>
    <v-card>
      <v-card-title>New Subscription</v-card-title>
      <v-card-subtitle v-if="isGithubRelease">
        Into golang? Try
        <span class="font-italic">golang/tools</span>
      </v-card-subtitle>
      <v-card-subtitle v-if="isRss">
        Into tech news? Try
        <span class="font-italic">theverge.com</span>
      </v-card-subtitle>
      <v-form @submit.prevent="search">
        <v-card-text class="pb-0">
          <v-row dense>
            <v-col cols="12">
              <v-select
                :items="types"
                v-model="type"
                :error-messages="typeErrors"
                label="Type"
                append-icon
                :disabled="types.length === 1"
              ></v-select>
            </v-col>
          </v-row>
          <v-row dense>
            <v-col cols="12">
              <v-text-field v-model.trim="name" :error-messages="nameErrors" :label="nameLabel"></v-text-field>
            </v-col>
          </v-row>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn type="submit" :loading="loading" class="primary">Search</v-btn>
          <v-snackbar v-model="snackbar" :top="true">
            Subscription added
            <v-btn text @click="snackbar = false">Close</v-btn>
          </v-snackbar>
        </v-card-actions>
      </v-form>
      <v-card-text>
        <div id="searchResults"></div>
        <v-row v-if="searchResults">
          <v-col cols="12">
            <v-list>
              <v-subheader>Please make a selection:</v-subheader>
              <v-list-item-group>
                <v-list-item
                  v-for="(channel, i) in searchResults"
                  :key="i+'-'+channel.id"
                  @click="openDialog(channel)"
                  :two-line="alreadySubscribed(channel)"
                  :disabled="alreadySubscribed(channel)"
                >
                  <v-list-item-icon>
                    <v-icon>{{channelIcon}}</v-icon>
                  </v-list-item-icon>
                  <v-list-item-content>
                    <v-list-item-title>{{channel.name}}</v-list-item-title>
                    <v-list-item-subtitle
                      v-if="alreadySubscribed(channel)"
                      class="red--text font-italic"
                    >Already subscribed</v-list-item-subtitle>
                  </v-list-item-content>
                </v-list-item>
              </v-list-item-group>
            </v-list>
          </v-col>
        </v-row>
        <v-dialog v-model="dialog" fullscreen hide-overlay>
          <v-card>
            <v-toolbar dark color="primary">
              <v-btn icon dark @click="dialog = false">
                <v-icon>{{ closeIcon }}</v-icon>
              </v-btn>
              <v-spacer></v-spacer>
              <v-toolbar-items>
                <v-btn dark text @click="subscribe">Subscribe</v-btn>
              </v-toolbar-items>
            </v-toolbar>
            <v-card-title v-if="selectedChannel">
              <v-icon class="mr-3">{{ channelIcon }}</v-icon>
              {{selectedChannel.name}}
            </v-card-title>
            <v-card-subtitle class="mt-3">How often do you want to receive digests?</v-card-subtitle>
            <v-card-text>
              <FrequencySelection v-model="frequency" />
            </v-card-text>
          </v-card>
        </v-dialog>
      </v-card-text>
    </v-card>
  </v-container>
</template>

<script>
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
import Features from "@/services/features.js";
import Api from "@/services/api.js";
import { mdiClose, mdiRss, mdiGithubCircle } from "@mdi/js";
import { mapGetters } from "vuex";
export default {
  components: {
    FrequencySelection
  },
  data() {
    return {
      closeIcon: mdiClose,
      githubIcon: mdiGithubCircle,
      rssIcon: mdiRss,

      twoRows: this.$vuetify.breakpoint.smAndDown,
      rssChannel: Features().rssChannel(),

      loading: false,
      snackbar: false,

      type: Features().rssChannel() ? "RSS" : "GithubRelease",
      typeErrors: [],

      name: "",
      nameErrors: [],

      frequency: {
        frequency: "Weekly",
        day: "Sat",
        time: "09:00:00"
      },

      searchResults: null,
      selectedChannel: null,
      dialog: null
    };
  },
  computed: {
    hasErrors() {
      return this.typeErrors.length == 0 && this.nameErrors.length == 0;
    },
    isGithubRelease() {
      return this.type === "GithubRelease";
    },
    isRss() {
      return this.type === "RssFeed";
    },
    nameLabel() {
      if (this.isGithubRelease) {
        return "Repository";
      } else if (this.isRss) {
        return "Url";
      } else {
        return "";
      }
    },
    channelIcon() {
      if (this.isGithubRelease) {
        return this.githubIcon;
      } else if (this.isRss) {
        return this.rssIcon;
      } else {
        return "";
      }
    },
    types() {
      let types = [];

      if (this.rssChannel) {
        types.push({ text: "Blog / RSS", value: "RssFeed" });
      }

      types.push({ text: "Github", value: "GithubRelease" });

      return types;
    },
    ...mapGetters(["subscriptions"])
  },
  methods: {
    search() {
      this.loading = true;

      if (this.validate()) {
        let params = { channel_type: this.type, query: this.name };
        Api()
          .get("subscriptions/search", { params: params })
          .then(resp => {
            this.loading = false;
            this.searchResults = resp.data.channels;
            if (resp.data.channels.length == 1) {
              this.openDialog(resp.data.channels[0]);
            } else {
              this.$vuetify.goTo("#searchResults");
            }
          })
          .catch(err => {
            this.loading = false;
            console.error(err);
          });
      }
    },
    openDialog(channel) {
      this.dialog = true;
      this.selectedChannel = channel;
    },
    alreadySubscribed(channel) {
      return this.subscriptions.some(sub => sub.channelId == channel.id);
    },
    async subscribe() {
      this.loading = true;
      if (this.validate()) {
        this.$store
          .dispatch("subscribe", {
            id: this.selectedChannel.id,
            frequency: this.frequency.frequency,
            day: this.frequency.day,
            time: this.frequency.time
          })
          .then(() => {
            this.loading = false;
            this.snackbar = true;
            this.name = "";
            this.searchResults = null;
            this.selectedChannel = null;
            this.dialog = null;
          })
          .catch(err => {
            this.loading = false;
            if (err.response.data.error) {
              this.nameErrors.push(err.response.data.error);
            } else {
              this.nameErrors.push("Something went wrong. Please try again.");
            }
          });
      } else {
        this.loading = false;
      }
    },
    validate() {
      this.clearErrors();
      if (this.isGithubRelease) {
        if (!/^[^/]+\/[^/]+$/.test(this.name)) {
          this.nameErrors.push("Format: author/repository");
        }
      } else if (this.isRss) {
        if (!/^.*[^.]+\.[^.]+.*$/.test(this.name)) {
          this.nameErrors.push("Format: theverge.com");
        }
      }
      return this.hasErrors;
    },
    clearErrors() {
      this.typeErrors = [];
      this.nameErrors = [];
    }
  }
};
</script>
