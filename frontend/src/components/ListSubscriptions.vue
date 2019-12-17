<template>
  <div>
    <v-container fluid>
      <v-row dense>
        <v-col v-for="subscription in subscriptions" :key="subscription.name" cols="12">
          <v-card color="blue" class="lighten-4">
            <div>
              <v-card-title>
                {{subscription.channelName}}
                <v-icon small class="pl-1">mdi-github-circle</v-icon>
              </v-card-title>
              <v-card-subtitle>
                <v-icon small>mdi-calendar</v-icon>
                {{subscription | showFrequency}}
              </v-card-subtitle>
            </div>
          </v-card>
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script>
import { mapState } from "vuex";
export default {
  computed: {
    ...mapState(["subscriptions"])
  },
  mounted() {
    this.$store.dispatch("loadSubscriptions");
  },
  filters: {
    showFrequency: function(sub) {
      var fmt = "every ";
      if (sub.frequency == "Daily") fmt += "day";
      else fmt += sub.day;
      fmt += " at ";
      fmt += sub.time;
      return fmt;
    }
  }
};
</script>
