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
            href="mailto:info@digester.app"
          >info@digester.app</a>
        </p>
      </v-card-subtitle>
      <v-card-text>
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
          <v-text-field v-model.trim="response" label="Enter the above text here"></v-text-field>
        </div>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          @click.stop="deleteAccount"
          :disabled="!showDeleteButton"
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
      console.log(newValue);
      if (newValue == true) {
        this.fetchChallenge();
      } else {
        this.challenge = null;
        this.response = null;
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
          // todo handle error
          this.failed = err;
        });
    },
    deleteAccount() {
      Api()
        .delete("/auth/me", { response: this.response })
        .then(() => {
          // todo show message
          this.go("/");
        })
        .catch(() => {
          // todo handle error
        });
    }
  }
};
</script>
