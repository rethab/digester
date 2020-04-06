<template>
  <v-container>
    <v-snackbar v-model="errorSnackbar" :top="true">
      {{errorMessage}}
      <v-btn text @click="errorSnackbar = false">Close</v-btn>
    </v-snackbar>
    <section id="searchInput">
      <Search :initialValue="searchInput" :loading="loading" v-on:search="search" />
    </section>
    <section id="searchResults">
      <ChannelSearchResults
        class="mt-6"
        v-if="searchResults"
        v-on:channelSelected="redirectToFrequencySubscriptionForm"
        :channels="searchResults"
        :channelType="searchChannelType"
        :searchError="searchError"
        :alreadyThere="alreadySubscribed"
        alreadyThereMessage="Already Subscribed"
      />
    </section>
  </v-container>
</template>

<script>
import Search from "@/components/subs/Search.vue";
import ChannelSearchResults from "@/components/channels/ChannelSearchResults.vue";
import Api from "@/services/api.js";
import { mapGetters } from "vuex";
export default {
  components: {
    Search,
    ChannelSearchResults
  },
  data() {
    return {
      loading: false,
      successSnackbar: false,
      errorSnackbar: false,
      errorMessage: null,

      searchInput: "",
      searchChannelType: null,
      searchResults: null,
      searchError: null
    };
  },
  computed: {
    ...mapGetters(["subscriptions"])
  },
  methods: {
    redirectToFrequencySubscriptionForm(channel) {
      this.$router.replace(`/subscribe/${channel.type}/${channel.id}`);
    },
    alreadySubscribed(channel) {
      return this.subscriptions.some(
        sub => sub.channelId == channel.id && sub.channelType == channel.type
      );
    },
    search(type, name) {
      this.searchResults = null;
      this.searchError = null;
      this.searchChannelType = type;
      this.searchInput = name;
      this.loading = true;

      let params = { channel_type: type, query: name };
      Api()
        .get("subscriptions/search", {
          params: params,
          timeout: 6000 // default timeout is 4s, but here we need to query upstream..
        })
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
            this.redirectToFrequencySubscriptionForm(this.searchResults[0]);
          } else {
            this.$vuetify.goTo("#searchResults");
          }
        })
        .catch(err => {
          this.loading = false;
          this.searchResults = [];
          if (err.response && err.response.data && err.response.data.error) {
            this.searchError = err.response.data.error;
          } else {
            this.searchError =
              "Something went wrong. Please make sure the input is valid. Contact us info@digester.app if you think this should be working.";
          }
        });
    }
  }
};
</script>
