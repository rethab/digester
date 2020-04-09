<template>
  <div>
    <TopSnackbar :message="snackbarMessage" v-model="topSnackbar" />
    <v-card flat>
      <v-card-title>
        <ChannelIcon :type="channel.type" class="mr-1" />
        <router-link
          style="text-decoration:none; color:inherit"
          class="ml-1"
          :to="channelLink"
        >{{channel.name}}</router-link>
      </v-card-title>
      <v-card-subtitle>You will get {{formatFrequency}} via E-Mail.</v-card-subtitle>
      <v-card-text class="pb-0">
        <FrequencySelection v-model="frequency" />
        <div v-if="!isAuthenticated">
          <v-text-field v-model.trim="email" :error-messages="emailErrors" label="E-Mail"></v-text-field>
        </div>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn to="/subs" color="secondary" outlined>Cancel</v-btn>
        <v-btn :loading="loading" @click="subscribe" color="primary">Subscribe</v-btn>
      </v-card-actions>
    </v-card>
  </div>
</template>

<script>
import Api from "@/services/api.js";
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import TopSnackbar from "@/components/common/TopSnackbar.vue";
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
import Channel from "@/models/Channel.js";
export default {
  components: {
    ChannelIcon,
    TopSnackbar,
    FrequencySelection
  },
  props: {
    channel: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      frequency: {
        frequency: "Weekly",
        day: "Sat",
        time: "09:00:00"
      },

      topSnackbar: null,
      snackbarMessage: "",

      loading: false,
      errorSnackbar: false,
      errorMessage: null,

      email: null,
      emailErrors: null
    };
  },
  computed: {
    isAuthenticated() {
      return this.$store.getters.isAuthenticated;
    },
    channelLink() {
      if (this.channel.type == Channel.List) {
        return `/list/${this.channel.id}`;
      } else {
        return this.channel.link;
      }
    },
    formatFrequency() {
      let time = this.frequency.time;
      time = time.substring(0, time.indexOf(":", 4));
      if (this.frequency.frequency == "Weekly") {
        let longDay = this.longDay(this.frequency.day);
        return `weekly updates on ${longDay} at ${time}`;
      } else {
        return `daily updates at ${time}`;
      }
    }
  },
  methods: {
    subscribe() {
      if (!this.validate()) {
        return;
      }

      this.loading = true;

      let subPromise = this.isAuthenticated
        ? this.subscribeUser()
        : this.subscribePending();

      subPromise
        .then(() => {
          this.loading = false;
          this.$router.replace("/subs");
        })
        .catch(err => {
          this.loading = false;

          this.topSnackbar = `subscription-add-fail-${this.channel.id}`;
          if (err.response.data.error) {
            this.snackbarMessage = err.response.data.error;
          } else {
            this.snackbarMessage = "Something went wrong. Please try again.";
          }
        });
    },
    subscribePending() {
      let payload = {
        email: this.email,
        // todo resolve via moment-timezone
        timezone: "Europe/Berlin",
        channelId: this.channel.id,
        channelType: this.channel.type,
        frequency: this.frequency.frequency,
        day: this.frequency.day,
        time: this.frequency.time
      };

      return new Promise((resolve, reject) => {
        Api()
          .post("subscriptions/add_pending", payload)
          .then(() => {
            resolve();
          })
          .catch(err => {
            reject(err);
          });
      });
    },
    subscribeUser() {
      let payload = {
        channelId: this.channel.id,
        channelType: this.channel.type,
        frequency: this.frequency.frequency,
        day: this.frequency.day,
        time: this.frequency.time
      };

      return new Promise((resolve, reject) => {
        this.$store
          .dispatch("subscribe", payload)
          .then(() => {
            resolve();
          })
          .catch(err => {
            reject(err);
          });
      });
    },
    validate() {
      if (!this.isAuthenticated) {
        if (!this.validateEmail()) {
          this.emailErrors = ["Please provide a valid e-mail address"];
          return false;
        }
      }
      return true;
    },
    validateEmail() {
      const blacklist = "`^*;,(){}[]";
      if (!blacklist.split("").every(b => !this.email.includes(b))) {
        return false;
      }

      let parts = this.email.split("@");
      if (parts.length != 2) {
        return false;
      }

      let name = parts[0],
        domain = parts[1];

      if (name.length == 0) {
        return false;
      }

      parts = domain.split(".");
      if (parts.length < 2) {
        return false;
      }

      if (!parts.every(p => p.length > 0)) {
        return false;
      }

      return true;
    },
    longDay(day) {
      let longDay = "";
      switch (day) {
        case "Mon":
          longDay = "Monday";
          break;
        case "Tue":
          longDay = "Tuesday";
          break;
        case "Wed":
          longDay = "Wednesday";
          break;
        case "Thu":
          longDay = "Thursday";
          break;
        case "Fri":
          longDay = "Friday";
          break;
        case "Sat":
          longDay = "Saturday";
          break;
        case "Sun":
          longDay = "Sunday";
          break;
      }
      return longDay;
    }
  }
};
</script>