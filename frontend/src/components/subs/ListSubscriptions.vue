<template>
  <div>
    <v-container fluid>
      <v-row dense>
        <v-col v-for="subscription in subscriptions" :key="subscription.name" cols="12">
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
  computed: {
    ...mapGetters(["subscriptions"])
  },
  mounted() {
    this.$store.dispatch("loadSubscriptions");
  }
};
</script>
