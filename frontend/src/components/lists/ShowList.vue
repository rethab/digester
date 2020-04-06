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
          <span v-if="value.channels.length == 0">This list contains no channels</span>
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
        <v-divider light class="mx-2"></v-divider>
        <v-card-actions>
          <div v-if="checkSubscription">
            <v-btn v-if="alreadySubscribed" :disabled="true" class="primary">Already Subscribed</v-btn>
            <v-btn v-else :to="`/subscribe/list/${value.id}`" class="primary">Subscribe</v-btn>
          </div>
          <v-spacer></v-spacer>
          <div v-if="isCreator">
            <router-link :to="`/list/${value.id}/edit`" text>
              <v-icon>{{ pencilIcon }}</v-icon>
            </router-link>
            <v-dialog v-model="deleteDialog" width="500">
              <template v-slot:activator="{ on }">
                <v-icon class="mr-1 ml-2" v-on="on" color="error lighten-1">{{ removeIcon }}</v-icon>
              </template>
              <v-card>
                <v-card-title>Are you sure?</v-card-title>
                <v-card-text>Please confirm that you want to delete this list.</v-card-text>
                <v-divider></v-divider>
                <v-card-actions>
                  <v-btn text @click="deleteDialog = false">Cancel</v-btn>
                  <v-spacer></v-spacer>
                  <v-btn @click="remove" :loading="deleteLoading" color="error" text>Confirm</v-btn>
                </v-card-actions>
              </v-card>
            </v-dialog>
          </div>
        </v-card-actions>
      </div>
    </v-card>
  </div>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import Channel from "@/models/Channel.js";
import { mdiPencilOutline, mdiDelete } from "@mdi/js";
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
      pencilIcon: mdiPencilOutline,
      removeIcon: mdiDelete,

      alreadySubscribed: false,
      deleteLoading: false,
      deleteDialog: null
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
      this.deleteLoading = true;
      this.$store
        .dispatch("deleteList", this.value)
        .then(() => {
          this.deleteLoading = false;
          this.deleteDialog = false;
        })
        .catch(() => {
          this.deleteLoading = false;
        });
      // todo error
    }
  }
};
</script>