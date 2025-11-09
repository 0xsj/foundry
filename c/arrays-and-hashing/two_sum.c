#include <stdlib.h>
#include <stdio.h>

// Simple hash map implementation for this problem
typedef struct {
    int key;
    int value;
    int used;
} HashEntry;

typedef struct {
    HashEntry* entries;
    int capacity;
} HashMap;

HashMap* createHashMap(int capacity) {
    HashMap* map = (HashMap*)malloc(sizeof(HashMap));
    map->capacity = capacity;
    map->entries = (HashEntry*)calloc(capacity, sizeof(HashEntry));
    return map;
}

void putHashMap(HashMap* map, int key, int value) {
    int index = abs(key) % map->capacity;
    while (map->entries[index].used && map->entries[index].key != key) {
        index = (index + 1) % map->capacity;
    }
    map->entries[index].key = key;
    map->entries[index].value = value;
    map->entries[index].used = 1;
}

int getHashMap(HashMap* map, int key, int* found) {
    int index = abs(key) % map->capacity;
    int start = index;
    
    do {
        if (!map->entries[index].used) {
            *found = 0;
            return -1;
        }
        if (map->entries[index].key == key) {
            *found = 1;
            return map->entries[index].value;
        }
        index = (index + 1) % map->capacity;
    } while (index != start);
    
    *found = 0;
    return -1;
}

void freeHashMap(HashMap* map) {
    free(map->entries);
    free(map);
}

int* twoSum(int* nums, int numsSize, int target, int* returnSize) {
    HashMap* map = createHashMap(numsSize * 2);
    int* result = (int*)malloc(2 * sizeof(int));
    
    for (int i = 0; i < numsSize; i++) {
        int complement = target - nums[i];
        int found;
        int index = getHashMap(map, complement, &found);
        
        if (found) {
            result[0] = index;
            result[1] = i;
            *returnSize = 2;
            freeHashMap(map);
            return result;
        }
        
        putHashMap(map, nums[i], i);
    }
    
    *returnSize = 0;
    freeHashMap(map);
    return result;
}