CREATE VIEW minor_table AS
SELECT
    id,
    data_value
FROM base_table
WHERE
    id < 10;