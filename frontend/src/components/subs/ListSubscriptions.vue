<template>
  <div>
    <div class="text-center">
      <h1 class="title mt-2" v-if="subscriptions.length > 0">Existing Subscriptions</h1>
    </div>
    <v-container fluid>
      <v-overlay absolute opacity="0.1" :value="loading">
        <v-progress-circular indeterminate color="primary" size="64"></v-progress-circular>
      </v-overlay>
      <v-row dense>
        <v-col
          v-for="subscription in subscriptions"
          :key="subscription.name"
          cols="12"
          :md="subscriptions.length > 1 ? 6 : 12"
        >
          <ShowSubscription :value="subscription" />
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script>
import { mapGetters } from "vuex";
import ShowSubscription from "@/components/subs/ShowSubscription.vue";
export default {
  components: {
    ShowSubscription
  },
  data() {
    return {
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
  }
};
</script>
