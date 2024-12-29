What parts can we abstract out?

- static resources are easiest

- with the power-user version the user supplies their own WGSL file or text and must ensure it is valid

- The results go to a resource that the user can use however they want
- The inputs are provided via a resource
- The whole system timing can be manually configured
- instead of entity population, iteration dimmensionality is specified.
- max num results is calculated via a callback based on dimmension sizes, or can be manually specified with the input data
