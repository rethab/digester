<template>
  <v-container>
    <v-card>
      <v-card-title>New Subscription</v-card-title>
      <v-card-subtitle>
        Into golang? Try
        <span class="font-italic">golang/tools</span>
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
              :disabled="true"
            ></v-select>
          </v-col>
          <v-col :cols="twoRows ? 8 : 5">
            <v-text-field
              v-model.trim="repository"
              :error-messages="repositoryErrors"
              :label="repositoryLabel"
            ></v-text-field>
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
        <v-btn @click.stop="subscribe" class="primary">Subscribe</v-btn>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script>
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
export default {
  components: {
    FrequencySelection
  },
  data() {
    return {
      twoRows: this.$vuetify.breakpoint.smAndDown,

      types: [{ text: "Github", value: "GithubRelease" }],
      type: "GithubRelease",
      typeErrors: [],

      repository: "",
      repositoryErrors: [],
      repositoryLabel: "Repository",

      frequency: {
        frequency: "Weekly",
        day: "Sat",
        time: "09:00:00"
      }
    };
  },
  computed: {
    hasErrors() {
      return this.typeErrors.length == 0 && this.repositoryErrors.length == 0;
    }
  },
  methods: {
    async subscribe() {
      if (this.validate()) {
        this.$store
          .dispatch("subscribe", {
            type: this.type,
            name: this.repository,
            frequency: this.frequency.frequency,
            day: this.frequency.day,
            time: this.frequency.time
          })
          .then(() => {
            this.repository = "";
          })
          .catch(err => {
            if (err.response.data.error) {
              this.repositoryErrors.push(err.response.data.error);
            } else {
              this.repositoryErrors.push(
                "Something went wrong. Please try again."
              );
            }
          });
      }
    },
    validate() {
      this.clearErrors();
      if (!/^[^/]+\/[^/]+$/.test(this.repository)) {
        this.repositoryErrors.push("Format: author/repository");
      }
      return this.hasErrors;
    },
    clearErrors() {
      this.typeErrors = [];
      this.repositoryErrors = [];
    }
  }
};
</script>
