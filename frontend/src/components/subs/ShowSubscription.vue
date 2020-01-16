<template>
  <v-card color="secondary" class="lighten-4">
    <div>
      <v-card-title>
        <a
          :href="channelLink"
          target="_blank"
          class="black--text"
          style="text-decoration:none"
        >{{value.channelName}}</a>
        <v-icon small class="pl-1">{{ channelIcon }}</v-icon>
      </v-card-title>
      <v-card-subtitle v-if="!editing">
        <v-icon small>{{ calendarIcon }}</v-icon>
        {{value | showFrequency}}
        <v-icon @click="editing = true" small class="ml-3">{{ pencilIcon }}</v-icon>
      </v-card-subtitle>
      <v-card-subtitle v-else>
        <FrequencySelection v-model="value" />
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn @click="editing = false" color="secondary" outlined>Cancel</v-btn>
          <v-btn @click.stop="save" class="primary">Save</v-btn>
        </v-card-actions>
      </v-card-subtitle>
    </div>
  </v-card>
</template>

<script>
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
import {
  mdiCalendar,
  mdiRss,
  mdiGithubCircle,
  mdiPencilOutline
} from "@mdi/js";
export default {
  components: {
    FrequencySelection
  },
  props: {
    value: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      editing: false,

      calendarIcon: mdiCalendar,
      githubIcon: mdiGithubCircle,
      rssIcon: mdiRss,
      pencilIcon: mdiPencilOutline
    };
  },
  computed: {
    isGithubRelease() {
      return this.value.channelType === "GithubRelease";
    },
    isRss() {
      return this.value.channelType === "RSS";
    },
    channelLink() {
      if (this.isGithubRelease) {
        return `https://github.com/${this.value.channelName}/releases`;
      } else {
        return this.channelName;
      }
    },
    channelIcon() {
      if (this.isGithubRelease) {
        return this.githubIcon;
      } else if (this.isRss) {
        return this.rssIcon;
      } else {
        return "";
      }
    }
  },
  methods: {
    save() {
      this.$store
        .dispatch("updateSubscription", {
          id: this.value.id,
          frequency: this.value.frequency,
          day: this.value.day,
          time: this.value.time
        })
        .then(() => {
          this.editing = false;
        });
    }
  },
  filters: {
    showFrequency: function(sub) {
      var fmt = "every ";
      if (sub.frequency == "Daily") fmt += "day";
      else fmt += sub.day;
      fmt += " at ";
      fmt += sub.time.substring(0, 5); // HH:MM
      return fmt;
    }
  }
};
</script>