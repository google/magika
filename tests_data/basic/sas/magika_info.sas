/* Create a dataset with Magika-related info */
data magika_info;
    length name $20 developer $20 type $40 feature $50;

    name = "Magika";
    developer = "Google";
    type = "AI File Detection";

    feature = "Supports 200+ file types"; output;
    feature = "High accuracy"; output;
    feature = "Fast processing"; output;
run;

/* Print the dataset */
proc print data=magika_info;
run;