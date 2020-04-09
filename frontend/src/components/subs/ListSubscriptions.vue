<template>
  <div>
    <TopSnackbar :message="snackbarMessage" v-model="topSnackbar" />
    <div class="text-center">
      <h1 class="title mt-2" v-if="subscriptions.length > 0">Existing Subscriptions</h1>
    </div>
    <v-container class="px-0" fluid>
      <v-overlay absolute opacity="0.1" :value="loading">
        <v-progress-circular indeterminate color="primary" size="64"></v-progress-circular>
      </v-overlay>
      <v-row>
        <v-col
          v-for="(subscription, idx) in subscriptions"
          :key="`${subscription.name}-${idx}`"
          cols="12"
          :md="subscriptions.length > 1 ? 6 : 12"
        >
          <ShowSubscription :value="subscription" v-on:remove="removeSubscription" />
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script>
import { mapGetters } from "vuex";
import TopSnackbar from "@/components/common/TopSnackbar.vue";
import ShowSubscription from "@/components/subs/ShowSubscription.vue";
export default {
  components: {
    TopSnackbar,
    ShowSubscription
  },
  data() {
    return {
      topSnackbar: null,
      snackbarMessage: "",

      loading: true,
      error: false
    };
  },
  computed: {
    ...mapGetters(["subscriptions"])
  },
  mounted() {
    this.$store
      .dispatch("loadSubscriptions")
      .then(() => {
        this.loading = false;
      })
      .catch(() => {
        this.loading = false;
        this.error = true;
      });
  },
  methods: {
    removeSubscription(sub) {
      this.$store
        .dispatch("deleteSubscription", sub.id)
        .then(() => {
          this.snackbarMessage = "Subscription deleted";
          this.topSnackbar = `subscription-deleted-${sub.id}`;
        })
        .catch(() => {
          this.snackbarMessage = "Failed to delete subscription";
          this.topSnackbar = `subscription-deleted-${sub.id}-fail`;
        });
    }
  }
};
</script>
