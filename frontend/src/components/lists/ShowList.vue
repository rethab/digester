<template>
  <div>
    <v-card color="secondary" class="lighten-4">
      <div>
        <v-card-title>
          <router-link
            style="text-decoration: none; color: inherit"
            :to="`/list/${value.id}`"
          >{{value.name}}</router-link>
        </v-card-title>
        <v-card-text>
          <p v-if="value.channels.length == 0">This list contains no channels</p>
          <span v-for="(channel, i) in value.channels" :key="i">
            <span v-if="i > 0">
              <span v-if="dense">&nbsp;|&nbsp;</span>
              <span v-else>
                <br />
              </span>
            </span>
            <ChannelIcon :type="channel.type" :small="true" class="mr-1" />
            <a
              style="text-decoration: none; color: inherit"
              target="_blank"
              :href="channel.link"
            >{{channel.name}}</a>
          </span>
        </v-card-text>
        <v-card-actions>
          <v-btn v-if="isCreator" @click="remove" class="error" text>Delete</v-btn>
          <v-btn v-if="isCreator" :to="`/list/${value.id}/edit`" class="secondary">Edit</v-btn>
          <v-spacer></v-spacer>
          <div v-if="checkSubscription">
            <v-btn v-if="alreadySubscribed" :disabled="true" class="primary">Already Subscribed</v-btn>
            <v-btn v-else :to="`/subscribe/list/${value.id}`" class="primary">Subscribe</v-btn>
          </div>
        </v-card-actions>
      </div>
    </v-card>
  </div>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import Channel from "@/models/Channel.js";
export default {
  components: {
    ChannelIcon
  },
  props: {
    value: {
      type: Object,
      required: true
    },
    dense: {
      type: Boolean,
      default: false
    },
    checkSubscription: {
      type: Boolean,
      require: true
    }
  },
  data() {
    return {
      alreadySubscribed: false
    };
  },
  computed: {
    isAuthenticated() {
      return this.$store.getters.isAuthenticated;
    },
    userId() {
      return this.$store.getters.userId;
    },
    isCreator() {
      return this.isAuthenticated && this.userId == this.value.creatorId;
    }
  },
  mounted() {
    if (this.isAuthenticated && this.checkSubscription) {
      this.$store.dispatch("loadSubscriptions").then(() => {
        this.alreadySubscribed = this.$store.getters.alreadySubscribed(
          Channel.List,
          this.value.id
        );
      });
    }
  },
  methods: {
    remove() {
      throw "Implement me";
    }
  }
};
</script>