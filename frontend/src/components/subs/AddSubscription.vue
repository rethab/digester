<template>
  <v-container>
    <v-snackbar v-model="successSnackbar" :top="true">
      Subscription added
      <v-btn text @click="successSnackbar = false">Close</v-btn>
    </v-snackbar>
    <v-snackbar v-model="errorSnackbar" :top="true">
      {{errorMessage}}
      <v-btn text @click="errorSnackbar = false">Close</v-btn>
    </v-snackbar>
    <section id="searchInput">
      <Search :initialValue="searchInput" :loading="loading" v-on:search="search" />
    </section>
    <section id="searchResults">
      <SearchResults
        class="mt-6"
        v-if="searchResults"
        v-on:openDialog="openDialog"
        :channels="searchResults"
        :channelType="searchChannelType"
      />
    </section>
    <FrequencyDialog
      v-on:closeDialog="closeDialog"
      v-on:subscribe="subscribe"
      v-if="selectedChannel"
      :channel="selectedChannel"
    />
  </v-container>
</template>

<script>
import Search from "@/components/subs/Search.vue";
import SearchResults from "@/components/subs/SearchResults.vue";
import FrequencyDialog from "@/components/subs/FrequencyDialog.vue";
import Api from "@/services/api.js";
import { mapGetters } from "vuex";
export default {
  components: {
    Search,
    SearchResults,
    FrequencyDialog
  },
  data() {
    return {
      loading: false,
      successSnackbar: false,
      errorSnackbar: false,
      errorMessage: null,

      searchInput: "foo",
      searchChannelType: null,
      searchResults: null,
      selectedChannel: null
    };
  },
  computed: {
    ...mapGetters(["subscriptions"])
  },
  methods: {
    openDialog(channel) {
      this.selectedChannel = channel;
    },
    closeDialog() {
      this.selectedChannel = null;
    },
    alreadySubscribed(channel) {
      return this.subscriptions.some(sub => sub.channelId == channel.id);
    },
    search(type, name) {
      this.searchResults = null;
      this.searchChannelType = type;
      this.searchInput = name;
      this.loading = true;

      let params = { channel_type: type, query: name };
      Api()
        .get("subscriptions/search", { params: params })
        .then(resp => {
          this.loading = false;
          this.searchResults = resp.data.channels;

          // make sure new ones appear first
          this.searchResults.sort((a, b) => {
            const aSub = this.alreadySubscribed(a),
              bSub = this.alreadySubscribed(b);

            if (!aSub && bSub) return -1;
            else if (!bSub && aSub) return 1;
            else 0;
          });

          if (
            this.searchResults.length == 1 &&
            !this.alreadySubscribed(this.searchResults[0])
          ) {
            this.openDialog(this.searchResults[0]);
          } else {
            this.$vuetify.goTo("#searchResults");
          }
        })
        .catch(() => {
          this.loading = false;
          this.searchResults = [];
        });
    },
    subscribe(channel, frequency) {
      this.$store
        .dispatch("subscribe", {
          id: channel.id,
          frequency: frequency.frequency,
          day: frequency.day,
          time: frequency.time
        })
        .then(() => {
          this.successSnackbar = true;
          this.selectedChannel = null;

          const allSubscribed = !this.searchResults.some(
            channel => !this.alreadySubscribed(channel)
          );

          if (allSubscribed) {
            this.searchResults = null;
            this.searchInput = "";
          }
        })
        .catch(err => {
          this.errorSnackbar = true;
          if (err.response.data.error) {
            this.errorMessage = err.response.data.error;
          } else {
            this.errorMessage = "Something went wrong. Please try again.";
          }
        });
    }
  }
};
</script>
