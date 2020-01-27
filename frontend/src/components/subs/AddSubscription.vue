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
      <v-card-text>
        <v-row cols="12" align="center">
          <v-col :cols="twoRows ? 4 : 3">
            <v-select
              :items="types"
              v-model="type"
              :error-messages="typeErrors"
              label="Type"
              append-icon
              :disabled="types.length === 1"
            ></v-select>
          </v-col>
          <v-col :cols="twoRows ? 8 : 5">
            <v-text-field v-model.trim="name" :error-messages="nameErrors" :label="nameLabel"></v-text-field>
          </v-col>
          <v-col v-if="!twoRows" cols="4">
            <FrequencySelection v-model="frequency" />
          </v-col>
        </v-row>
        <v-row v-if="twoRows">
          <v-col cols="12">
            <FrequencySelection v-model="frequency" />
          </v-col>
        </v-row>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn @click.stop="subscribe" :loading="loading" class="primary">Subscribe</v-btn>
        <v-snackbar v-model="snackbar" :top="true">
          Subscription added
          <v-btn text @click="snackbar = false">Close</v-btn>
        </v-snackbar>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script>
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
import Features from "@/services/features.js";
export default {
  components: {
    FrequencySelection
  },
  data() {
    return {
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
      }
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
      return this.type === "RSS";
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
    types() {
      let types = [];

      if (this.rssChannel) {
        types.push({ text: "Blog / RSS", value: "RSS" });
      }

      types.push({ text: "Github", value: "GithubRelease" });

      return types;
    }
  },
  methods: {
    async subscribe() {
      this.loading = true;
      if (this.validate()) {
        this.$store
          .dispatch("subscribe", {
            type: this.type,
            name: this.name,
            frequency: this.frequency.frequency,
            day: this.frequency.day,
            time: this.frequency.time
          })
          .then(() => {
            this.loading = false;
            this.snackbar = true;
            this.name = "";
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
