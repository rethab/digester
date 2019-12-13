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
        <v-btn @click.stop="subscribe" class="success">Subscribe</v-btn>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script>
import FrequencySelection from "@/components/FrequencySelection.vue";
export default {
  components: {
    FrequencySelection
  },
  data() {
    return {
      twoRows: false, // this.$vuetify.breakpoint.smAndDown,

      types: ["Github"],
      type: "Github",
      typeErrors: [],

      repository: "",
      repositoryErrors: [],
      repositoryLabel: "Repository",

      frequency: {
        frequency: "weekly",
        day: "Sat",
        hour: "09:00"
      }
    };
  },
  computed: {
    hasErrors() {
      return this.typeErrors.length == 0 && this.repositoryErrors.length == 0;
    }
  },
  methods: {
    subscribe() {
      if (this.validate()) {
        // todo plug backend
        var fmt = "every ";
        if (this.frequency.frequency == "daily") fmt += "day";
        else fmt += this.frequency.day;
        fmt += " at ";
        fmt += this.frequency.hour;
        this.$store.dispatch("subscribe", {
          type: this.type,
          name: this.repository,
          frequency: fmt
        });
        this.repository = "";
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
