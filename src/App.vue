<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "vue3-toastify";
import { VideoMetaData } from "./type";

const loading = ref(true)

const videos_metadata = ref<VideoMetaData[]>([])

const dir_path = ref<String[]>([])

onMounted(async () => {
  try {
    dir_path.value.push("/run/media/sam/film/marvel")
    await invoke("create_table_app");
    await invoke("sync_app_files", { root: "/run/media/sam/film/", apiKey: "4c602a26" });
    videos_metadata.value = await invoke<VideoMetaData[]>("get_all_video_metadata_app");
    toast.success("Saved successfully!", {
      toastClassName: "toast"
    });
  } catch (e) {
    console.error("Error during initialization:", e);
  }
  finally {
    loading.value = false
  }
})


import { open } from '@tauri-apps/plugin-dialog';

function add_dir() {
  open({
    multiple: false,
    directory: true,
  }).then(async (dir) => {
    if (!dir)
      return

    dir_path.value.push(dir)
    await invoke("sync_app_files", { root: dir, apiKey: "4c602a26" });
    videos_metadata.value = await invoke<VideoMetaData[]>("get_all_video_metadata_app");
    toast.success("Saved successfully!", {
      toastClassName: "toast"
    });
  }).catch(e => console.log(e))

}

</script>

<template>
  <main class="container  mx-auto">
    <button class="btn btn-block" @click="add_dir">
      add file
    </button>
    <div v-if="loading">
      <span class="loading loading-dots loading-xl"></span>
    </div>
    <div v-else class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-3">
      <div class="card bg-base-100 w-full image-full" v-for="(movie, i) in videos_metadata" :key="i">
        <figure>
          <img class="w-full" :src="movie.imdb_metadata?.poster" alt="Shoes" />
        </figure>
        <div class="card-body">
          <h2 class="card-title">
            {{ movie.imdb_metadata?.title || movie.name }}
            <span class="badge badge-primary" v-if="movie.imdb_metadata?.year || movie.year">
              {{ movie.imdb_metadata?.year || movie.year }}
            </span>
            <span class="badge badge-secondary" v-if="movie.imdb_metadata?.imdb_rating && movie.imdb_metadata?.imdb_rating != 'N/A'">
              {{ movie.imdb_metadata?.imdb_rating }}
            </span>
          </h2>
          <p>{{ movie.imdb_metadata?.plot }}</p>
          <div class="card-actions justify-end">
            <span v-if="movie.series">
            s{{ movie.series.season }}
            e{{ movie.series.episode }}
          </span>
          <span v-if="movie.files_data.length > 1">
            <p v-for="path in movie.files_data" :key="path.path">
              {{ path.path }}
            </p>
          </span>

            <!-- <button class="btn btn-primary">edit movie name</button> -->
          </div>


          
        </div>
      </div>

    </div>
  </main>
</template>
