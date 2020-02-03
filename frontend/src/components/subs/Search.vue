<template>
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
    <v-form @submit.prevent="submit">
      <v-card-text class="pb-0">
        <v-row dense>
          <v-col cols="12">
            <v-select
              :items="types"
              v-model="type"
              :error-messages="typeErrors"
              label="Type"
              append-icon
              :disabled="types.length === 1"
            ></v-select>
          </v-col>
        </v-row>
        <v-row dense>
          <v-col cols="12">
            <v-text-field v-model.trim="name" :error-messages="nameErrors" :label="nameLabel"></v-text-field>
          </v-col>
        </v-row>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn type="submit" :loading="loading" class="primary">Search</v-btn>
      </v-card-actions>
    </v-form>
  </v-card>
</template>

<script>
export default {
  props: {
    loading: {
      type: Boolean,
      required: true
    },
    initialValue: {
      type: String,
      required: true
    }
  },
  data() {
    return {
      snackbar: false,

      type: "RssFeed",
      types: [
        { text: "Blog / News", value: "RssFeed" },
        { text: "Github", value: "GithubRelease" }
      ],
      typeErrors: [],

      name: this.initialValue,
      nameErrors: [],

      searchResults: null,
      selectedChannel: null,
      dialog: null
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
      return this.type === "RssFeed";
    },
    nameLabel() {
      if (this.isGithubRelease) {
        return "Repository";
      } else if (this.isRss) {
        return "Url";
      } else {
        return "";
      }
    }
  },
  watch: {
    initialValue(newValue) {
      this.name = newValue;
    }
  },
  methods: {
    submit() {
      this.clearErrors();
      if (this.validate()) {
        this.$emit("search", this.type, this.name);
      }
    },
    validate() {
      if (this.isGithubRelease) {
        // if a user enters a github url, we help them a bit by
        // extracting the repository
        let repoWithUrl = /^.*github\.com.*\/([^/]+\/[^/]+)$/.exec(this.name);
        if (repoWithUrl) {
          this.name = repoWithUrl[1];
        }
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