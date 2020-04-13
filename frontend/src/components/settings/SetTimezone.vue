<template>
  <v-container>
    <TopSnackbar message="Settings Updated" v-model="topSnackbar" />
    <v-card>
      <v-card-title>Timezone</v-card-title>
      <v-card-subtitle>We need to know your timezone in order to send the digests at the correct time relative to your location.</v-card-subtitle>
      <v-card-text>
        <v-autocomplete
          label="Please select a Timezone"
          v-model="timezone"
          :items="timezones"
          :error-messages="timezoneErrors"
          :menu-props="autocompleteMenuProps"
        ></v-autocomplete>
        <v-overlay absolute opacity="0.1" :value="loading">
          <v-progress-circular indeterminate color="primary" size="64"></v-progress-circular>
        </v-overlay>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn @click.stop="save" class="primary">Save</v-btn>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script>
import TopSnackbar from "@/components/common/TopSnackbar.vue";
import momentTZ from "moment-timezone";
export default {
  components: {
    TopSnackbar
  },
  data() {
    return {
      loading: true,

      topSnackbar: null,

      condensedView: this.$vuetify.breakpoint.smAndDown,

      timezones: momentTZ.tz.names(),
      timezone: null,
      timezoneErrors: []
    };
  },
  computed: {
    hasErrors() {
      return this.timezoneErrors.length == 0;
    },
    autocompleteMenuProps() {
      /* vuetify-autocomplete doesn't render very well on mobile:
        The dropdown overlaps with the input field. This is a known
        issue and there is a workaround, however, implementing the
        workaround took too much time and I think we could do this later:
        https://github.com/vuetifyjs/vuetify/issues/5950
        So for now we have our own workaround, which is to show only a very
        dropdown (that expands upwards). */

      // default properties copied from the vuetify-autocomplete docs
      let defaultProps = {
        closeOnClick: false,
        closeOnContentClick: false,
        disableKeys: true,
        openOnClick: false,
        maxHeight: 304
      };

      if (this.condensedView) {
        defaultProps.maxHeight = 200;
        defaultProps.top = true;
      }
      return defaultProps;
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
            this.topSnackbar = true;
          })
          .catch(err => {
            if (err.response.data.error) {
              this.timezoneErrors.push(err.response.data.error);
            } else {
              this.timezoneErrors.push(
                "Something went wrong. Please try again."
              );
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
