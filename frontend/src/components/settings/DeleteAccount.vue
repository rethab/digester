<template>
  <v-container>
    <v-card>
      <v-card-title>Danger Zone</v-card-title>
      <v-card-subtitle>
        <p>If you really don't like us anymore, you can delete your account here.</p>
        <p>
          Be aware, however, that deleting your account means your account is
          <i>actually</i> gone. We will delete all your subscriptions.
        </p>
        <p>
          Is there anything we can do to make you change your decision? Is something not working for you? Are you missing a feature? Please contact us at
          <a
            class="black--text"
            href="mailto:info@digester.app"
          >info@digester.app</a>
        </p>
      </v-card-subtitle>
      <v-card-text>
        <p v-if="errorMessage" class="error">{{errorMessage}}</p>
        <v-checkbox v-model="knowWhatTheyreDoing">
          <template v-slot:label>
            <span class="body-2">I know what I'm doing and I want to leave.</span>
          </template>
        </v-checkbox>
        <div v-if="!!challenge">
          <p>
            Fair enough. There's only one thing we need you to do before you leave. Please type
            <strong>{{challenge}}</strong> into the below textfield
          </p>
          <v-text-field
            v-model.trim="response"
            :error-messages="deleteErrors"
            label="Enter the above text here"
          ></v-text-field>
        </div>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          @click.stop="deleteAccount"
          :disabled="!showDeleteButton"
          :loading="deletionInProgress"
          class="red"
        >Delete Account Forever</v-btn>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script>
import Api from "@/services/api.js";
export default {
  components: {},
  data() {
    return {
      knowWhatTheyreDoing: false,
      deletionInProgress: false,
      errorMessage: null,
      deleteErrors: [],
      challenge: null,
      response: null
    };
  },
  computed: {
    showDeleteButton() {
      return this.challenge && this.challenge == this.response;
    }
  },
  watch: {
    knowWhatTheyreDoing(newValue) {
      this.deleteErrors = [];
      if (newValue == true) {
        this.fetchChallenge();
      } else {
        this.challenge = null;
        this.response = null;
        this.errorMessage = null;
      }
    }
  },
  methods: {
    fetchChallenge() {
      Api()
        .get("/auth/delete_challenge")
        .then(resp => {
          this.challenge = resp.data.challenge;
        })
        .catch(err => {
          this.errorMessage =
            "Something went wrong. Please try again or contact support if this problem persists.";
          this.failed = err;
        });
    },
    deleteAccount() {
      this.deletionInProgress = true;
      Api()
        .delete("/auth/me", { data: { response: this.response } })
        .then(() => {
          this.deletionInProgress = false;

          // redirecting to auth login, because there we can show messages
          // easily. the UX is bad though ("Login Required" title), but there
          // are more important things than UX for leaving customers after
          // their account deletion was successful..
          this.$store.dispatch("unauthenticated").then(() => {
            this.$router.push({
              name: "auth-login",
              query: { accountDeleted: true }
            });
          });
        })
        .catch(() => {
          this.deleteErrors.push(
            "Failed to delete account. Please try again or contact support if the problem persists."
          );
          this.deletionInProgress = false;
        });
    }
  }
};
</script>
