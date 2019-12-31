<template>
  <v-container>
    <v-card>
      <v-card-title>Timezone</v-card-title>
      <v-card-subtitle>We need to know your timezone in order to send the digests at the correct time relative to your location.</v-card-subtitle>
      <v-card-text>
        <v-autocomplete
          label="Please select a Timezone"
          v-model="timezone"
          :items="timezones"
          :error-messages="timezoneErrors"
        ></v-autocomplete>
        <v-overlay absolute opacity="0.1" :value="loading">
          <v-progress-circular indeterminate color="primary" size="64"></v-progress-circular>
        </v-overlay>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn @click.stop="save" class="success">Save</v-btn>
      </v-card-actions>
      <v-snackbar v-model="snackbar" timeout="2000" top="true">
        Settings updated
        <v-btn dark text @click="snackbar = false">Close</v-btn>
      </v-snackbar>
    </v-card>
  </v-container>
</template>

<script>
import momentTZ from "moment-timezone";
export default {
  components: {},
  data() {
    return {
      loading: true,
      snackbar: false,

      timezones: momentTZ.tz.names(),
      timezone: null,
      timezoneErrors: []
    };
  },
  computed: {
    hasErrors() {
      return this.timezoneErrors.length == 0;
    }
  },
  mounted() {
    this.$store.dispatch("loadTimezone").then(resp => {
      this.loading = false;
      if (resp.data.timezone) {
        this.timezone = resp.data.timezone;
      }
    });
  },
  methods: {
    async save() {
      if (this.validate()) {
        this.$store
          .dispatch("setTimezone", this.timezone)
          .then(() => {
            this.snackbar = true;
          })
          .catch(err => {
            if (err.response.data.error) {
              this.timezoneErrors.push(err.response.data.error);
            } else {
              // todo
            }
          });
      }
    },
    validate() {
      this.clearErrors();
      if (!this.timezone) {
        this.timezoneErrors.push("Please select a timezone");
      }
      return this.hasErrors;
    },
    clearErrors() {
      this.timezoneErrors = [];
    }
  }
};
</script>
