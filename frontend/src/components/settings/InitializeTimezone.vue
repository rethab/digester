<template>
  <v-container>
    <ErrorCard
      v-if="timezoneFailed"
      title="Failed to set Timezone :("
      message="We have failed to automatically determine your timezone. Please click on the button below
        to manually set your timezone.
        <br />Having a timezone is cruicial in order to send the digests at the correct time."
      to="/settings"
    />
    <v-snackbar :value="timezoneSet" :top="true" :multi-line="true">
      We have automatically set your timezone to {{timezone}}.
      <br />You can adjust this in the settings.
      <v-btn dark text @click="timezoneSet = false">Dismiss</v-btn>
    </v-snackbar>
  </v-container>
</template>

<script>
import momentTZ from "moment-timezone";
import ErrorCard from "@/components/common/ErrorCard.vue";
export default {
  components: {
    ErrorCard
  },
  data() {
    return {
      timezone: null,

      timezoneSet: false,
      timezoneFailed: false
    };
  },
  mounted() {
    const timezone = momentTZ.tz.guess();
    if (timezone) {
      this.$store
        .dispatch("setTimezone", timezone)
        .then(() => {
          this.timezoneSet = true;
          this.timezone = timezone;
          this.removeFirstLogin();
        })
        .catch(() => {
          this.timezoneFailed = true;
          this.removeFirstLogin();
        });
    }
  },
  methods: {
    removeFirstLogin() {
      let query = Object.assign({}, this.$route.query);
      delete query.firstLogin;
      this.$router.replace({ query });
    }
  }
};
</script>
