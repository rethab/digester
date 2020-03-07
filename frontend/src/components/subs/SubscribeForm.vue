<template>
  <v-container>
    <v-snackbar v-model="successSnackbar" :top="true">
      Subscription added
      <v-btn text @click="successSnackbar = false">Close</v-btn>
    </v-snackbar>
    <v-snackbar v-model="errorSnackbar" :top="true">
      {{errorMessage}}
      <v-btn text @click="errorSnackbar = false">Close</v-btn>
    </v-snackbar>
    <v-card>
      <v-card-title>
        <ChannelIcon :type="channel.type" class="mr-1" />Subscribe to
        <router-link
          style="text-decoration:none; color:inherit"
          class="ml-1"
          :to="channelLink"
        >{{channel.name}}</router-link>
      </v-card-title>
      <v-card-subtitle>You will get {{formatFrequency}} via E-Mail.</v-card-subtitle>
      <v-card-text>
        <FrequencySelection v-model="frequency" />
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn :loading="loading" @click="subscribe" class="primary">Subscribe</v-btn>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
import Channel from "@/models/Channel.js";
export default {
  components: {
    ChannelIcon,
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

      loading: false,
      successSnackbar: false,
      errorSnackbar: false,
      errorMessage: null
    };
  },
  computed: {
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
      this.loading = true;
      let payload = {
        channel: this.channel,
        frequency: this.frequency.frequency,
        day: this.frequency.day,
        time: this.frequency.time
      };

      this.$store
        .dispatch("subscribe", payload)
        .then(() => {
          this.successSnackbar = true;
          this.loading = false;
        })
        .catch(err => {
          this.errorSnackbar = true;
          this.loading = false;
          if (err.response.data.error) {
            this.errorMessage = err.response.data.error;
          } else {
            this.errorMessage = "Something went wrong. Please try again.";
          }
        });
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